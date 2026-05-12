use crate::models::config::NcbiConfig;
use crate::models::context::PipelineContext;
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct EsearchResponse {
    esearchresult: EsearchResult,
}

#[derive(Debug, Deserialize)]
struct EsearchResult {
    idlist: Vec<String>,
}

pub fn find_ref_genome(context: &mut PipelineContext, config: &NcbiConfig) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    let mut query = format!(
        "\"{}\"[Organism] AND srcdb_refseq[PROP] AND biomol_genomic[PROP]",
        context.source
    );

    let src_ids = search_entrez(&client, &query, &config.api_key, "nuccore")?;

    if src_ids.is_empty() {
        return Err(anyhow!("No reference genome found for {}", context.source));
    }

    context.src_genome_ids = src_ids;

    query = format!(
        "\"{}\"[Organism] AND srcdb_refseq[PROP] AND biomol_genomic[PROP]",
        context.target
    );

    let tgt_ids = search_entrez(&client, &query, &config.api_key, "nuccore")?;

    if tgt_ids.is_empty() {
        return Err(anyhow!("No reference genome found for {}", context.target));
    }

    context.tgt_genome_ids = tgt_ids;

    Ok(())
}

pub fn fetch_genome_fasta(
    context: &mut PipelineContext,
    config: &NcbiConfig,
    genome_dir: &PathBuf,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();

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
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&tgt_dir)?;

    for id in &context.src_genome_ids {
        let file_path = src_dir.join(format!("{}.fa", id));

        if file_path.exists() {
            continue;
        }

        let fasta_text = fetch_genbank(&client, id, &config.api_key, "nuccore", "fasta", "text")?;
        fs::write(&file_path, fasta_text)?;
    }

    for id in &context.tgt_genome_ids {
        let file_path = tgt_dir.join(format!("{}.fa", id));

        if file_path.exists() {
            continue;
        }

        let fasta_text = fetch_genbank(&client, id, &config.api_key, "nuccore", "fasta", "text")?;
        fs::write(&file_path, fasta_text)?;
    }

    Ok(())
}

pub fn fetch_genome_xml(
    context: &mut PipelineContext,
    config: &NcbiConfig,
    genome_dir: &PathBuf,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();

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
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&tgt_dir)?;

    for id in &context.src_genome_ids {
        let file_path = src_dir.join(format!("{}.xml", id));

        if file_path.exists() {
            continue;
        }

        let xml_text = fetch_genbank(
            &client,
            id,
            &config.api_key,
            "nuccore",
            "gbwithparts",
            "xml",
        )?;
        fs::write(&file_path, xml_text)?;
    }

    for id in &context.tgt_genome_ids {
        let file_path = tgt_dir.join(format!("{}.xml", id));

        if file_path.exists() {
            continue;
        }

        let xml_text = fetch_genbank(
            &client,
            id,
            &config.api_key,
            "nuccore",
            "gbwithparts",
            "xml",
        )?;
        fs::write(&file_path, xml_text)?;
    }

    Ok(())
}

fn search_entrez(
    client: &reqwest::blocking::Client,
    query: &str,
    api_key: &str,
    db: &str,
) -> Result<Vec<String>> {
    let mut request = client
        .get("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi")
        .query(&[("db", db), ("term", query), ("retmode", "json")]);

    if !api_key.trim().is_empty() {
        request = request.query(&[("api_key", api_key.trim())]);
    }

    let response = request.send().context("Network error during esearch")?;
    let search_data: EsearchResponse =
        response.json().context("JSON parse error during esearch")?;

    Ok(search_data.esearchresult.idlist)
}

fn fetch_genbank(
    client: &reqwest::blocking::Client,
    id: &str,
    api_key: &str,
    db: &str,
    rettype: &str,
    retmode: &str,
) -> Result<String> {
    let mut request = client
        .get("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi")
        .query(&[
            ("db", db),
            ("id", id),
            ("rettype", rettype),
            ("retmode", retmode),
        ]);

    if !api_key.trim().is_empty() {
        request = request.query(&[("api_key", api_key.trim())]);
    }

    let raw_text = request
        .send()
        .context("Network error during efetch")?
        .text()
        .context("Text parse error during efetch")?;

    Ok(raw_text)
}
