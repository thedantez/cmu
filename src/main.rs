mod vk_api;

#[tokio::main]
async fn main() {
    let client = vk_api::VkClient::new(TOKEN.to_string());
    match client.get_dialogs().await {
        Ok(dialogs) => {
            for d in dialogs {
                println!("{}: {}\n", d.peer_id, d.title);
            }
        }
        Err(e) => eprintln!("error: {}", e),
    }
}
