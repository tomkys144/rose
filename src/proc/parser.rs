use crate::models::context::PipelineContext;
use crate::models::protein::{MatchResult, Protein};
use anyhow::{Result, anyhow};
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;

pub fn find_similar(context: &mut PipelineContext, genome_dir: &PathBuf) -> Result<()> {
    if context.src_genome_ids.is_empty() {
        return Err(anyhow!("No source genome IDs in context!"));
    }

    if context.tgt_genome_ids.is_empty() {
        return Err(anyhow!("No target genome IDs in context!"));
    }

    let safe_src_name = context.source.replace(" ", "_");
    let safe_tgt_name = context.target.replace(" ", "_");
    let src_dir = genome_dir.join(safe_src_name);
    let tgt_dir = genome_dir.join(safe_tgt_name);
    if !src_dir.exists() {
        return Err(anyhow!("Source genome directory not found!"));
    }
    if !tgt_dir.exists() {
        return Err(anyhow!("Target genome directory not found!"));
    }

    let mut src_proteins: HashMap<String, Protein> = HashMap::new();
    let mut tgt_proteins: HashMap<String, Protein> = HashMap::new();
    let mut matches: HashMap<(String, String), Vec<MatchResult>> = HashMap::new();

    for id in &context.src_genome_ids {
        let file_path = src_dir.join(format!("{}.xml", id));
        if !file_path.exists() {
            return Err(anyhow!("Source genome XML file not found!"));
        }

        let (parsed_src, parsed_tgt, parsed_matches) = parse_xml(&file_path)?;

        insert_protein(&mut src_proteins, parsed_src);
        insert_protein(&mut tgt_proteins, parsed_tgt);

        for (key, match_res) in parsed_matches {
            matches.entry(key).or_default().push(match_res);
        }
    }

    for id in &context.tgt_genome_ids {
        let file_path = tgt_dir.join(format!("{}.xml", id));
        if !file_path.exists() {
            return Err(anyhow!("Target genome XML file not found!"));
        }

        let (parsed_tgt, parsed_src, parsed_matches) = parse_xml(&file_path)?;

        insert_protein(&mut tgt_proteins, parsed_tgt);
        insert_protein(&mut src_proteins, parsed_src);

        for (key, match_res) in parsed_matches {
            matches.entry(key).or_default().push(match_res);
        }
    }

    context.src_proteome = src_proteins;
    context.tgt_proteome = tgt_proteins;
    context.results = matches;

    Ok(())
}

fn parse_xml(
    file_path: &PathBuf,
) -> Result<(
    Vec<Protein>,
    Vec<Protein>,
    HashMap<(String, String), MatchResult>,
)> {
    let mut reader = Reader::from_file(file_path)?;
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut src_proteins: Vec<Protein> = Vec::new();
    let mut tgt_proteins: Vec<Protein> = Vec::new();
    let mut matches: HashMap<(String, String), MatchResult> = HashMap::new();

    let mut current_protein: Option<Protein> = None;
    let mut current_chromosome: Option<String> = None;

    let mut current_tag = String::new();
    let mut feature_key = String::new();
    let mut qualifier_name = String::new();
    let mut feature_location = String::new();
    let mut current_tgt_uniprot: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(anyhow!(
                    "XML Error at position {}: {:?}",
                    reader.buffer_position(),
                    e
                ));
            }

            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_tag = tag_name.clone();

                if tag_name == "GBFeature" {
                    feature_key.clear();
                    feature_location.clear();
                    current_tgt_uniprot = None;
                } else if tag_name == "GBQualifier" {
                    qualifier_name.clear();
                }
            }
            Ok(Event::Text(e)) => {
                let text = String::from_utf8_lossy(&e).into_owned();

                match current_tag.as_str() {
                    "GBFeature_key" => {
                        feature_key = text.clone();

                        if feature_key == "CDS" {
                            current_protein = Some(Protein {
                                chromosome: current_chromosome.clone(),
                                ..Default::default()
                            });
                        }
                    }
                    "GBFeature_location" => {
                        if feature_key == "CDS" {
                            if let Some(prot) = &mut current_protein {
                                prot.position = parse_pos(&text);
                            }
                        }
                    }
                    "GBQualifier_name" => qualifier_name = text.clone(),
                    "GBQualifier_value" => {
                        if feature_key == "source" && qualifier_name == "chromosome" {
                            current_chromosome = Some(text.clone());
                        } else if feature_key == "CDS" {
                            if let Some(prot) = &mut current_protein {
                                match qualifier_name.as_str() {
                                    "locus_tag" | "gene" => {
                                        if prot.gene_id.is_empty() {
                                            prot.gene_id = text;
                                        }
                                    }
                                    "translation" => prot.sequence = Some(text),
                                    "note" => {
                                        prot.note = Some(text.clone());

                                        if let Some((_, right)) = text.split_once("uniprot|") {
                                            let id = right
                                                .split(|c: char| {
                                                    c.is_whitespace()
                                                        || c == ','
                                                        || c == ';'
                                                        || c == ')'
                                                })
                                                .next()
                                                .unwrap_or(right);

                                            current_tgt_uniprot = Some(id.to_string());
                                        }
                                    }
                                    "db_xref" => {
                                        if text.starts_with("UniProtKB") {
                                            if let Some(id) = text.split(':').last() {
                                                prot.uniprot_id = Some(id.to_string());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "GBFeature" && feature_key == "CDS" {
                    if let Some(mut prot) = current_protein.take() {
                        if prot.gene_id.is_empty() {
                            prot.gene_id = "N/A".to_string();
                        }

                        let src_id = prot.primary_id();

                        if let Some(tgt_id) = current_tgt_uniprot.take() {
                            tgt_proteins.push(Protein {
                                gene_id: "N/A".to_string(),
                                uniprot_id: Some(tgt_id.clone()),
                                ..Default::default()
                            });

                            matches.insert(
                                (src_id.clone(), tgt_id.clone()),
                                MatchResult {
                                    src: src_id.clone(),
                                    tgt: tgt_id,
                                    score: 0,
                                    identity: 0.0,
                                },
                            );
                        }

                        src_proteins.push(prot);
                    }
                }
                current_tag.clear();
            }
            _ => {}
        }
        buf.clear();
    }

    Ok((src_proteins, tgt_proteins, matches))
}

fn parse_pos(loc: &str) -> Option<(u32, u32)> {
    let loc_str = if let Some((_, right)) = loc.split_once(':') {
        right
    } else {
        loc
    };

    let mut numbers = Vec::new();
    let mut current_num = String::new();

    for c in loc_str.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if !current_num.is_empty() {
            if let Ok(num) = current_num.parse::<u32>() {
                numbers.push(num);
            }
            current_num.clear();
        }
    }

    if !current_num.is_empty() {
        if let Ok(num) = current_num.parse::<u32>() {
            numbers.push(num);
        }
    }

    if numbers.is_empty() {
        return None;
    }

    let start = *numbers.iter().min()?;
    let end = *numbers.iter().max()?;

    Some((start, end))
}

fn insert_protein(map: &mut HashMap<String, Protein>, vec: Vec<Protein>) {
    for prot in vec {
        let primary = prot.primary_id();

        if let Some(existing_prot) = map.get_mut(&primary) {
            existing_prot.merge_none_fields(prot);
            continue;
        }

        if prot.uniprot_id.is_some()
            && prot.gene_id != "N/A"
            && prot.gene_id != "UNKNOWN"
            && prot.gene_id != "UNKNOWN_UNTIL_PARSED"
        {
            if let Some(mut old_placeholder) = map.remove(&prot.gene_id) {
                old_placeholder.merge_none_fields(prot);

                map.insert(primary, old_placeholder);
                continue;
            }
        }

        map.insert(primary, prot);
    }
}
