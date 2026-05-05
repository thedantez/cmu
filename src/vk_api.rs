//  api calls to api.vk.com

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
    pub async fn get_dialogs(&self) -> Result<Vec<Dialog>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.vk.com/method/messages.getConversations?access_token={}&v=5.199&count=3",
            self.token
        );
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let items = resp["response"]["items"]
            .as_array()
            .ok_or("not found response.items")?;
        let mut dialogs = Vec::new();
        for item in items {
            let peer_id = item["conversation"]["peer"]["id"]
                .as_i64()
                .ok_or("not found peer.id")?;
            let title = if let Some(chat_settings) = item["conversation"]["chat_settings"].as_object() {
                chat_settings.get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("w/out title")
            } else {
                "myselft chat"
            };
            let dialog = Dialog {
                peer_id,
                title: title.to_string(),
            };
            dialogs.push(dialog);
        }
        Ok(dialogs)
    }
}

pub struct Dialog {
    pub title: String,
    pub peer_id: i64,
}

pub struct Message {
    pub sender_name: String,
    pub text: String,
}
