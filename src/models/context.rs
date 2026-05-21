use crate::models::protein::{MatchResult, Protein};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct PipelineContext {
    pub source: String,
    pub target: String,

    pub src_genome_ids: Vec<String>,
    pub tgt_genome_ids: Vec<String>,

    pub src_proteome: HashMap<String, Protein>,
    pub tgt_proteome: HashMap<String, Protein>,

    pub results: HashMap<(String, String), Vec<MatchResult>>,
}

impl PipelineContext {
    pub(crate) fn clear(&mut self) {
        self.source.clear();
        self.target.clear();
        self.src_genome_ids.clear();
        self.tgt_genome_ids.clear();
        self.src_proteome.clear();
        self.tgt_proteome.clear();
        self.results.clear();
    }
}
