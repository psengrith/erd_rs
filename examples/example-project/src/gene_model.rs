use crate::organism_model::OrganismModel;
use std::default::Default;

pub struct GeneModel {
    id: u32,
    pub name: String,
    pub nucleotide_5_end: String,
    #[doc = "`#[relation = n..n : exist_in]`"]
    pub organism: OrganismModel,
}

impl GeneModel {
    pub fn new(id: u32, name: String, nucleotide_5_end: String, organism: OrganismModel) -> Self {
        Self {
            id,
            name,
            nucleotide_5_end,
            organism,
        }
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub(self) fn summarized(&self) -> String {
        format!("{}:{}", self.name, self.organism.nomenclature)
    }

    pub(crate) fn summarized2(&self) -> String {
        format!("{}:{}", self.name, self.nucleotide_5_end)
    }
}

impl Default for GeneModel {
    fn default() -> Self {
        Self {
            id: Default::default(),
            name: Default::default(),
            nucleotide_5_end: Default::default(),
            organism: Default::default(),
        }
    }
}
