pub struct WebSocketServer;
impl WebSocketServer {
    pub fn new(_p: u16) -> Self {
        Self
    }
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
