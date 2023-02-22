#[derive(serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub file: String,
    pub position: u32,
}
