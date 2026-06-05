// VK API client implementation
use reqwest;
use serde_json::Value;
use urlencoding;
use crate::client::{Client, Message, Dialog};
use async_trait::async_trait;

const VK_API_VERSION: &str = "5.199";
const VK_API_BASE: &str = "https://api.vk.com/method";
const DEFAULT_DIALOGS_COUNT: u32 = 40;

pub struct VkClient {
    token: String,
    user: Value,
    client: reqwest::Client,
}

impl VkClient {
    pub async fn new(token: String) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = format!(
            "{}/users.get?access_token={}&v={}",
            VK_API_BASE, token, VK_API_VERSION
        );
        let resp = client.get(&url).send().await?.json::<Value>().await?;
        
        if resp["error"].is_object() {
            return Err("VK API: unable to retrieve user data".into());
        }

        Ok(VkClient {
            token,
            user: resp["response"][0].clone(),
            client,
        })
    }

    fn build_api_url(method: &str, params: &[(&str, String)]) -> String {
        let params_str = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        
        format!(
            "{}/{}?{}&v={}",
            VK_API_BASE, method, params_str, VK_API_VERSION
        )
    }

    fn extract_name_from_profile(profile: &Value) -> String {
        let first_name = profile["first_name"].as_str().unwrap_or("");
        let last_name = profile["last_name"].as_str().unwrap_or("");
        format!("{} {}", first_name, last_name).trim().to_string()
    }

    fn get_dialog_title(
        item: &Value,
        profiles: &[Value],
        groups: &[Value],
        peer_id: i64,
    ) -> String {
        if let Some(chat_settings) = item["conversation"]["chat_settings"].as_object() {
            chat_settings
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Untitled Chat".to_string())
        } else if peer_id > 0 && peer_id < 2_000_000_000 {
            // Private chat
            profiles
                .iter()
                .find(|p| p["id"].as_i64() == Some(peer_id))
                .map(Self::extract_name_from_profile)
                .unwrap_or_else(|| format!("User {}", peer_id))
        } else if peer_id < 0 {
            // Community/group chat
            let abs_id = -peer_id;
            groups
                .iter()
                .find(|g| g["id"].as_i64() == Some(abs_id))
                .and_then(|g| g["name"].as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Group {}", abs_id))
        } else {
            "Unknown Chat".to_string()
        }
    }
}

#[async_trait]
impl Client for VkClient {
    fn get_user(&self) -> &Value {
        &self.user
    }

    async fn send_message(
        &self,
        peer_id: i64,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let encoded_text = urlencoding::encode(text);
        let params = vec![
            ("access_token", self.token.clone()),
            ("peer_id", peer_id.to_string()),
            ("message", encoded_text.to_string()),
            ("random_id", "0".to_string()),
        ];

        let url = format!(
            "{}/messages.send?{}",
            VK_API_BASE,
            params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&")
        );

        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        
        if resp["error"].is_object() {
            return Err("VK API: failed to send message".into());
        }
        Ok(())
    }

    async fn get_dialogs(
        &self,
    ) -> Result<Vec<Dialog>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/messages.getConversations?access_token={}&count={}&extended=1&v={}",
            VK_API_BASE, self.token, DEFAULT_DIALOGS_COUNT, VK_API_VERSION
        );
        
        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let items = resp["response"]["items"]
            .as_array()
            .ok_or("VK API: missing 'response.items' in response")?;

        let profiles = resp["response"]["profiles"]
            .as_array()
            .map(|a| a.clone())
            .unwrap_or_default();
        let groups = resp["response"]["groups"]
            .as_array()
            .map(|a| a.clone())
            .unwrap_or_default();

        let mut dialogs = Vec::new();
        for item in items {
            let peer_id = item["conversation"]["peer"]["id"]
                .as_i64()
                .ok_or("VK API: missing 'peer.id' in response")?;

            let title = Self::get_dialog_title(item, &profiles, &groups, peer_id);

            dialogs.push(Dialog {
                peer_id,
                title,
            });
        }

        Ok(dialogs)
    }

    async fn get_messages(
        &self,
        peer_id: i64,
        count: u32,
    ) -> Result<Vec<Message>, Box<dyn std::error::Error + Send + Sync>> {
        let url = format!(
            "{}/messages.getHistory?access_token={}&peer_id={}&count={}&extended=1&v={}",
            VK_API_BASE, self.token, peer_id, count, VK_API_VERSION
        );

        let resp = self.client.get(&url).send().await?.json::<Value>().await?;
        let items = resp["response"]["items"]
            .as_array()
            .ok_or("VK API: missing 'response.items' in response")?;

        let profiles = resp["response"]["profiles"]
            .as_array()
            .map(|a| a.clone())
            .unwrap_or_default();
        let groups = resp["response"]["groups"]
            .as_array()
            .map(|a| a.clone())
            .unwrap_or_default();

        let user_id = self.user["id"].as_i64().unwrap_or(0);

        let mut messages = Vec::new();
        for item in items {
            let from_id = item["from_id"].as_i64().unwrap_or(0);
            let text = item["text"].as_str().unwrap_or("").to_string();
            let is_me = from_id == user_id;

            let (sender_name, owner) = if from_id > 0 {
                // Personal user
                profiles
                    .iter()
                    .find(|p| p["id"].as_i64() == Some(from_id))
                    .map(|profile| {
                        (
                            Self::extract_name_from_profile(profile),
                            profile.clone(),
                        )
                    })
                    .unwrap_or_else(|| (format!("User {}", from_id), Value::Null))
            } else if from_id < 0 {
                // Group/community
                let abs_id = -from_id;
                groups
                    .iter()
                    .find(|g| g["id"].as_i64() == Some(abs_id))
                    .map(|group| {
                        (
                            group["name"].as_str().unwrap_or("Group").to_string(),
                            group.clone(),
                        )
                    })
                    .unwrap_or_else(|| (format!("Group {}", abs_id), Value::Null))
            } else {
                (format!("id{}", from_id), Value::Null)
            };

            messages.push(Message {
                is_me,
                owner,
                sender_name,
                text,
            });
        }

        messages.reverse();
        Ok(messages)
    }
}
