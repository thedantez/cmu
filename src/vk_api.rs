use reqwest;
use serde_json::Value;

pub struct VkClient {
    token: String,
    client: reqwest::Client,
}

impl VkClient {
    pub fn new(token: String) -> Self {
        VkClient {
            token,
            client: reqwest::Client::new(),
        }
    }
    pub async fn get_dialogs(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.vk.com/method/messages.getConversations?access_token={}&v=5.199&count=3",
            self.token
        );
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        println!("answer from vk: {}", resp);
        Ok(())
    }
}

struct Dialog {
    pub title: String,
    pub peer_id: i64,
}

struct Message {
    pub sender_name: String,
    pub text: String,
}
