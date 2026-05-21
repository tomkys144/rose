use crate::models::context::PipelineContext;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UniprotResponse {
    results: Vec<UniprotEntry>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct UniprotEntry {
    primaryAccession: String,
    sequence: SequenceData,
}

#[derive(Debug, Deserialize)]
struct SequenceData {
    value: String,
}

pub fn fetch_missing_sequences(ctx: &mut PipelineContext) -> Result<usize> {
    let client = reqwest::blocking::Client::new();

    let mut missing_ds = Vec::new();
    for (_, prot) in &ctx.tgt_proteome {
        if prot.gene_id == "N/A" {
            missing_ds.push(prot.uniprot_id.clone().unwrap());
        }
    }

    if missing_ds.is_empty() {
        return Ok(0);
    }

    let mut fetched_cnt = 0;

    for chunk in missing_ds.chunks(50) {
        let accessions = chunk.join(",");
        let url = format!(
            "https://rest.uniprot.org/uniprotkb/accessions?accessions={}",
            accessions
        );

        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .context("Failed to fetch Uniprot sequences")?;

        if !response.status().is_success() {
            continue;
        }

        let data: UniprotResponse = response.json().context("Failed to parse Uniprot response")?;

        for entry in data.results {
            if let Some(tgt_protein) = ctx.tgt_proteome.values_mut().find(|p| {
                p.uniprot_id.as_deref() == Some(&entry.primaryAccession)
            }) {
                tgt_protein.sequence = Some(entry.sequence.value);
                fetched_cnt += 1;
            }
        }
    }

    Ok(fetched_cnt)
}
