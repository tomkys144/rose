use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct NcbiConfig {
    pub api_key: String,
    pub max_retries: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseConfig {
    pub gap_open: i32,
    pub gap_extend: i32,
    pub min_identity: f32,
    pub score_matrix: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlignConfig {
    pub gap_open: i32,
    pub gap_extend: i32,
    pub min_identity: f32,
    pub score_matrix: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStep {
    FindReferenceGenome(NcbiConfig),
    FetchGenomeFasta(NcbiConfig),
    FetchGenomeAnnotations(NcbiConfig),
    ParseXmlAnnotations(),

    FetchMissingUniprot(),
    AlignFound(ParseConfig),

    FetchTargetProteome,
    RunAlignment(AlignConfig),
    ParallelBranches(Vec<Vec<PipelineStep>>),
}

#[derive(Debug, Clone, Default)]
pub struct PipelineConfig {
    pub steps: Vec<PipelineStep>,
}
