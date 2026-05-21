use crate::models::config::{AlignConfig, ParseConfig};
use crate::models::context::PipelineContext;
use crate::models::protein::{MatchResult, Protein};
use anyhow::{Result, anyhow};
use bio::alignment::pairwise::{Aligner, MatchFunc};
use bio::alignment::sparse::{find_kmer_matches, hash_kmers, lcskpp};
use bio::alignment::{Alignment, AlignmentOperation};
use bio::scores::{blosum30, blosum45, blosum62};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

pub fn align_all(context: &mut PipelineContext, cfg: AlignConfig) -> Result<()> {
    let mtrx: fn(u8, u8) -> i32 = match cfg.score_matrix.as_str() {
        "Blosum 62" => blosum62 as fn(u8, u8) -> i32,
        "Blosum 45" => blosum45 as fn(u8, u8) -> i32,
        "Blosum 30" => blosum30 as fn(u8, u8) -> i32,
        _ => return Err(anyhow!("Invalid score matrix!")),
    };

    let k: usize = 5;

    let pre_computed: Vec<(&String, HashSet<&[u8]>)> = context
        .tgt_proteome
        .par_iter()
        .filter_map(|(tgt_id, tgt)| {
            let seq = tgt.sequence.as_ref()?;
            let bytes = seq.as_bytes();
            if bytes.len() < k {
                return None;
            }

            let mut unique_kmers = HashSet::new();
            for kmer in bytes.windows(k) {
                unique_kmers.insert(kmer);
            }

            Some((tgt_id, unique_kmers))
        })
        .collect();

    let mut global_index: HashMap<&[u8], Vec<&String>> = HashMap::new();

    for (tgt_id, unique_kmers) in pre_computed {
        for kmer in unique_kmers {
            global_index.entry(kmer).or_default().push(tgt_id);
        }
    }

    let all_matches: Vec<((String, String), MatchResult)> = context
        .src_proteome
        .par_iter()
        .flat_map_iter(|(src_id, src)| {
            let mut local_res = Vec::new();

            if let Some(src_seq) = &src.sequence {
                let src_bytes = src_seq.as_bytes();
                let mut candidate_hits: HashMap<&String, usize> = HashMap::new();

                if src_bytes.len() > k {
                    let mut seen_kmers = HashSet::new();
                    for kmer in src_bytes.windows(k) {
                        if seen_kmers.insert(kmer) {
                            if let Some(hit_targets) = global_index.get(kmer) {
                                for tgt_id in hit_targets {
                                    *candidate_hits.entry(tgt_id).or_default() += 1;
                                }
                            }
                        }
                    }
                }

                for (tgt_id, hits) in candidate_hits {
                    let tgt_bytes = context
                        .tgt_proteome
                        .get(tgt_id)
                        .unwrap()
                        .sequence
                        .as_ref()
                        .unwrap()
                        .as_bytes();

                    let max_len = src_bytes.len().max(tgt_bytes.len()) as f32;

                    let max_possible_identity = ((hits * k) as f32 / max_len) * 100.0;

                    if max_possible_identity < cfg.min_identity {
                        continue;
                    }

                    let matches = find_kmer_matches(src_bytes, tgt_bytes, k);
                    let sparse_al = lcskpp(&matches, k);

                    let identity = (sparse_al.score as f32 / max_len) * 100.0;

                    if identity >= cfg.min_identity * 0.9 {
                        local_res.push((
                            (src_id.clone(), tgt_id.clone()),
                            MatchResult {
                                src: src_id.clone(),
                                tgt: tgt_id.clone(),
                                score: sparse_al.score as i32,
                                identity: identity,
                            },
                        ))
                    }
                }
            }
            local_res
        })
        .collect();

    for (key, match_res) in all_matches {
        context.results.entry(key).or_default().push(match_res);
    }
    Ok(())
}

pub fn align_matches(context: &mut PipelineContext, cfg: ParseConfig) -> Result<()> {
    let mtrx: &fn(u8, u8) -> i32 = match cfg.score_matrix.as_str() {
        "Blosum 62" => &(blosum62 as fn(u8, u8) -> i32),
        "Blosum 45" => &(blosum45 as fn(u8, u8) -> i32),
        "Blosum 30" => &(blosum30 as fn(u8, u8) -> i32),
        _ => return Err(anyhow!("Invalid score matrix!")),
    };
    let mut aligner = Aligner::new(cfg.gap_open, cfg.gap_extend, mtrx);

    for match_res in context.results.values_mut() {
        match_res.retain_mut(|res| {
            let src = context.src_proteome.get(&res.src).unwrap();
            let tgt = context.tgt_proteome.get(&res.tgt).unwrap();

            let (score, identity) = score_alignment(src, tgt, &mut aligner);

            res.score = score;
            res.identity = identity;

            identity >= cfg.min_identity
        })
    }

    Ok(())
}

fn score_alignment<F: MatchFunc>(
    src: &Protein,
    tgt: &Protein,
    aligner: &mut Aligner<F>,
) -> (i32, f32) {
    let src_seq = match &src.sequence {
        Some(seq) => seq,
        None => return (i32::MIN, 0.0),
    };
    let tgt_seq = match &tgt.sequence {
        Some(seq) => seq,
        None => return (i32::MIN, 0.0),
    };

    let alignment = aligner.global(src_seq.as_bytes(), tgt_seq.as_bytes());
    let identity = calc_identity(&alignment);
    (alignment.score, identity)
}

fn calc_identity(alignment: &Alignment) -> f32 {
    let mut matches = 0;
    let mut total = 0;

    for op in &alignment.operations {
        match op {
            AlignmentOperation::Match => {
                matches += 1;
                total += 1;
            }
            AlignmentOperation::Subst => {
                total += 1;
            }
            AlignmentOperation::Del => {
                total += 1;
            }
            AlignmentOperation::Ins => {
                total += 1;
            }
            _ => {
                continue;
            }
        }
    }

    if total == 0 {
        0.0
    } else {
        (matches as f32 / total as f32) * 100.0
    }
}
