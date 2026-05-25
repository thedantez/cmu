use crate::client::{Client, Dialog, Message};
use rand::thread_rng;
use rand::Rng;
use async_trait::async_trait;


const PLACEHOLDERS: [&str; 10] = ["Привет", "Пока", "Как дела", "Пошел посру", "Прикинь", "Есть темка", "Удачи", "Что это", "Давай вместе", "Ну не знаю"];

pub struct TestClient {
    dialog_count: u16,
    user: serde_json::Value,
    messages: Vec<Message>,
}

impl TestClient {
    pub fn new(_: String) -> Self {
        TestClient {
            dialog_count: 40,
            user: serde_json::from_str("{}").unwrap(),
            messages: Self::generate_messages(400),
        }
    }

    fn generate_messages(count: u32) -> Vec<Message> {
        let placeholders = PLACEHOLDERS.map(String::from);
        let mut messages = Vec::new();
        let mut rng = thread_rng();
        for _ in 1..count {
            let text = &placeholders[rng.gen_range(0..placeholders.len())];
            messages.push(Message {
                is_me: false,
                owner: serde_json::from_str("{}").unwrap(),
                sender_name: "Kolyan".to_string(),
                text: text.to_string(),
            });
        }
        messages.reverse();
        messages
    }
}

#[async_trait]
impl Client for TestClient {
    fn get_user(&self) -> &serde_json::Value {&self.user}

    async fn get_dialogs(&self) -> Result<Vec<Dialog>, Box<dyn std::error::Error + Send + Sync>> {
        let mut dialogs = Vec::new();
        for i in 1..self.dialog_count {
            let dialog = Dialog {
                peer_id: 0,
                title: format!("Test Chat {}", i),
            };
            dialogs.push(dialog);
        }
        Ok(dialogs)
    }

    async fn get_messages(&self, _: i64, _: u32) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.messages.clone())
    }

    async fn send_message(&self, _: i64, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Sended a message: {}", &text);
        Ok(())
    }

//    async fn auth(&self) {}
}


