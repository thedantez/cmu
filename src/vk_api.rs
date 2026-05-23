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
    pub fn new(token: String) -> Self {
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
            "https://api.vk.com/method/messages.getConversations?access_token={}&v=5.199&count=40&extended=1",
            self.token
        );
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let items = resp["response"]["items"]
            .as_array()
            .ok_or("not found response.items")?;
        let empty_ar = Vec::new();
        let profiles = resp["response"]["profiles"].as_array().unwrap_or(&empty_ar);
        let groups = resp["response"]["groups"].as_array().unwrap_or(&empty_ar);

        let mut dialogs = Vec::new();
        for item in items {
            let peer_id = item["conversation"]["peer"]["id"]
                .as_i64()
                .ok_or("not found peer.id")?;
            let title = if let Some(chat_settings) = item["conversation"]["chat_settings"].as_object() {
                chat_settings.get("title") // it's group chat
                    .and_then(|v| v.as_str())
                    .unwrap_or("w/out title").to_string()
            } else if peer_id > 0 && peer_id < 2000000000 { // it's private chat
                if let Some(profile) = profiles.iter().find(|p| p["id"].as_i64() == Some(peer_id)) {
                    format!("{} {}", profile["first_name"].as_str().unwrap_or(""), profile["last_name"].as_str().unwrap_or(""))
                } else {
                    format!("User {}", peer_id)
                }
            } else if peer_id < 0 {  // it's different chat: comunity
                let abs_id = -peer_id;
                if let Some(group) = groups.iter().find(|g| g["id"].as_i64() == Some(abs_id)) {
                    group["name"].as_str().unwrap_or("Group").to_string()
                } else {
                    format!("Group {}", abs_id)
                }
            } else {
                "Unknown chat".to_string()
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

        let empty_ar = Vec::new();
        let profiles = resp["response"]["profilies"].as_array().unwrap_or(&empty_ar);
        let groups = resp["response"]["groups"].as_array().unwrap_or(&empty_ar);

        let mut messages = Vec::new();
        // parsing
        for item in items {
            //let from_id = item["from_id"].ok_or("not found message")?;
            let from_id = item["from_id"].as_i64().unwrap_or(0);
            let text = item["text"].as_str().unwrap_or("").to_string();

            let mut sender_name = format!("id{}", from_id);

            if from_id > 0 {
                if let Some(profile) = profiles.iter().find(|p| p["id"].as_i64() == Some(from_id)) {
                    let f_name = profile["first_name"].as_str().unwrap_or("");
                    let l_name = profile["last_name"].as_str().unwrap_or("");
                    sender_name = format!("{} {}", f_name, l_name);
                }
            } else if from_id < 0 {
                let abs_id = -from_id;
                if let Some(group) = groups.iter().find(|g| g["id"].as_i64() == Some(abs_id)) {
                    sender_name = group["name"].as_str().unwrap_or("Group").to_string();
                }
            }

            messages.push(Message {
                sender_name: sender_name,
                text
            });
        }
        messages.reverse();
        Ok(messages)
    }
}
