#[derive(Debug, Clone)]
pub struct Protein {
    pub uniprot_id: String,
    pub gene_id: String,
    pub sequence: String,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub query: String,
    pub target: String,
    pub score: f32,
    pub identity: f32,
}