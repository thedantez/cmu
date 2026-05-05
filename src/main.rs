mod vk_api;
use vk_api::VkClient;

#[tokio::main]
async fn main() {
    let token = "vk1.a.1wOvxYOFAFijemO8VNUkDl1SjwPagbbzz71kSvtdNQmimc8GFEZIAwHt1Lgkwdz72gR6rLgpqyalY0UUcAUz6hGU5-8bGFjkMGhCSEIvhx9rvoR1SPuq51Br02lmGE9LDZ25Vxb5GOgRfucpMiRsd2QK2_Iy0shPZfPgwJOEDzSMJGaovLKZ_JmHwJeBvcXpK3xAwPx-iy3I_1P6MF6gjw";
    let client = VkClient::new(token.to_string());
    client.get_dialogs().await.unwrap();
}
