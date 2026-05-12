use crate::models::protein::{MatchResult, Protein};

#[derive(Debug, Clone, Default)]
pub struct PipelineContext {
    pub source: String,
    pub target: String,

    pub src_genome_ids: Vec<String>,
    pub tgt_genome_ids: Vec<String>,

    pub results: Vec<MatchResult>
}