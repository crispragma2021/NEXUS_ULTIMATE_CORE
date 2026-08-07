pub struct MeshManager;
impl MeshManager {
    pub fn new() -> Result<Self, anyhow::Error> {
        Ok(Self)
    }
    pub async fn start_discovery(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
    pub async fn register_node(&self, _p: u16) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
