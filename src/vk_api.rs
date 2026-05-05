use reqwest;
use serde_json::Value;

struct VkClient {
    const TOKEN: &str = "vk1.a.1wOvxYOFAFijemO8VNUkDl1SjwPagbbzz71kSvtdNQmimc8GFEZIAwHt1Lgkwdz72gR6rLgpqyalY0UUcAUz6hGU5-8bGFjkMGhCSEIvhx9rvoR1SPuq51Br02lmGE9LDZ25Vxb5GOgRfucpMiRsd2QK2_Iy0shPZfPgwJOEDzSMJGaovLKZ_JmHwJeBvcXpK3xAwPx-iy3I_1P6MF6gjw";
    let url = format!(
        "https://api.vk.com/method/messages.getConversations?access_token={}&v=5.199&count=3",
        TOKEN
    );
    let resp = reqwest::get(&url).await?.json::<Value>().await?;
}

struct Dialog {
    title: &str = "";
    url: &str = "";
}

struct Message {
    owner: &str = "";
    text: &str = "";
}
