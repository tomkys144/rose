#[derive(Debug, Clone, Default)]
pub struct Protein {
    pub gene_id: String,
    pub uniprot_id: Option<String>,
    pub sequence: Option<String>,
    pub chromosome: Option<String>,
    pub position: Option<(u32, u32)>,
    pub note: Option<String>,
}

impl Protein {
    pub fn primary_id(&self) -> String {
        self.uniprot_id
            .clone()
            .unwrap_or_else(|| self.gene_id.clone())
    }
    
    pub fn merge_none_fields(&mut self, mut other: Protein) {
        if self.gene_id == "N/A" || self.gene_id == "UNKNOWN" {
            self.gene_id = other.gene_id;
        }
        self.uniprot_id = self.uniprot_id.take().or(other.uniprot_id);
        self.sequence = self.sequence.take().or(other.sequence);
        self.chromosome = self.chromosome.take().or(other.chromosome);
        self.position = self.position.take().or(other.position);
        self.note = self.note.take().or(other.note);
    }
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub src: String,
    pub tgt: String,
    pub score: i32,
    pub identity: f32,
}
