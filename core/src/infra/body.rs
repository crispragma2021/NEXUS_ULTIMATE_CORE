pub enum HealthLevel {
    Critical,
    Stressed,
    Optimal,
}
pub struct BodyAwareness {
    pub health: HealthLevel,
}
impl BodyAwareness {
    pub async fn snapshot(&self) -> Self {
        Self {
            health: HealthLevel::Optimal,
        }
    }
}
