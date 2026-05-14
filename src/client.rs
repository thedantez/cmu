use async_trait::async_trait;

pub struct Dialog {
    pub title: String,
    pub peer_id: i64,
}

#[derive(Clone)]
pub struct Message {
    pub sender_name: String,
    pub text: String,
}


#[async_trait]
pub trait Client: Send + Sync {
    async fn get_dialogs(&self) -> Result<Vec<Dialog>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_messages(&self, peer_id: i64, count: u32) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>>;
    async fn send_message(&self, peer_id: i64, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn auth(&self);
}
