#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fursona {
    pub species: String,
    pub body_type: String,
    pub markings: String,
    pub accessories: Vec<String>,
    pub personality: String,
}

impl Fursona {
    pub fn new(
        species: String,
        body_type: String,
        markings: String,
        accessories: Vec<String>,
        personality: String,
    ) -> Self {
        Self {
            species,
            body_type,
            markings,
            accessories,
            personality,
        }
    }
}
