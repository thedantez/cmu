//  api calls to api.vk.com
use reqwest; use serde_json::Value;
use urlencoding;
use crate::client::{Client, Message, Dialog};
use async_trait::async_trait;


pub struct VkClient {
    token: String,
    client: reqwest::Client,
}

impl VkClient {
    fn new(token: String) -> Self {
        VkClient {
            token,
            client: reqwest::Client::new(),
        }
    }

}


#[async_trait]
impl Client for VkClient {
    async fn send_message(&self, peer_id: i64, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let encoded_text = urlencoding::encode(&text);
        let url = format!(
            "https://api.vk.com/method/messages.send?access_token={}&v=5.199&peer_id={}&message={}&random_id=0",
            self.token,
            peer_id,
            encoded_text
        );
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        if resp["error"].is_object() {
            Err("VK sending message error".into())
        } else {
            Ok(())
        }
    }

    async fn get_dialogs(&self) -> Result<Vec<Dialog>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.vk.com/method/messages.getConversations?access_token={}&v=5.199&count=40",
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
                "Default chat"
            };
            let dialog = Dialog {
                peer_id,
                title: title.to_string(),
            };
            dialogs.push(dialog);
        }
        Ok(dialogs)
    }

    async fn get_messages(&self, peer_id: i64, count: u32) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "https://api.vk.com/method/messages.getHistory?access_token={}&v=5.199&peer_id={}&count={}",
            self.token,
            peer_id,
            count,
        );
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let items = resp["response"]["items"]
            .as_array()
            .ok_or("not found response.items")?;
        let mut messages = Vec::new();
        // parsing
        for item in items {
            // let sender_name = item["from_id"].ok_or("not found message")?;
            let text = item["text"].as_str().unwrap_or("").to_string();
            messages.push(Message {
                sender_name: "companion".to_string(),
                text
            });
        }
        messages.reverse();
        Ok(messages)
    }

    async fn auth(&self) {
        // TODO: Создать аутентификацию для вк
        println!("Auth is not implemented!")
    }
}
