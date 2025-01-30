#[derive(Default)]
pub struct OrganismModel {
    id: u32,
    pub nomenclature: String,
}

impl OrganismModel {
    pub fn new(id: u32, nomenclature: String) -> Self {
        Self { id, nomenclature }
    }

    fn log_to_console(&self) {
        println!("{}:{}", self.id, self.nomenclature);
    }
}
