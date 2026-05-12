use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct NcbiConfig {
    pub api_key: String,
    pub max_retries: usize
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlignConfig {
    pub min_identity: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineStep {
    FindReferenceGenome(NcbiConfig),
    FetchGenomeFasta(NcbiConfig),
    FetchGenomeAnnotations(NcbiConfig),

    ParseXmlAnnotations,

    FetchTargetProteome,
    RunAlignment(AlignConfig),
    ParallelBranches(Vec<Vec<PipelineStep>>),
}

#[derive(Debug, Clone, Default)]
pub struct PipelineConfig {
    pub steps: Vec<PipelineStep>,
}