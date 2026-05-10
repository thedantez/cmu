pub struct Dialog {
    pub title: String,
    pub peer_id: i64,
}

#[derive(Clone)]
pub struct Message {
    pub sender_name: String,
    pub text: String,
}


pub trait Client {
    fn new(token: String) -> Self;
    async fn get_dialogs(&self) -> Result<Vec<Dialog>, Box<dyn std::error::Error>>;
    async fn get_messages(&self, peer_id: i64, count: u32) -> Result<Vec<Message>, Box<dyn std::error::Error>>;
    async fn send_message(&self, peer_id: i64, text: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn auth(&self);
}
