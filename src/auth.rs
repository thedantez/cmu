use std::io::{self, Write};
use url::Url;

pub async fn get_access_token() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let auth_url = "https://oauth.vk.com/authorize?\
                    client_id=6287487&\
                    display=page&\
                    redirect_uri=https://oauth.vk.com/blank.html&\
                    scope=messages,offline&\
                    response_type=token&\
                    v=5.199";

    println!("Opening browser for authorization...");

    std::thread::spawn(move || {
        let _ = open::that(auth_url);
    });

    print!("\ninput url after auth here: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Err("Error: value \"input\" is empty".into());
    }

    let target_str = if input.contains('#') {
        input.replace('#', "?")
    } else if input.contains("access_token=") {
        format!("http://localhost/?{}", input)
    } else {
        input.to_string()
    };
    let parsed = Url::parse(&target_str)?;

    let token = parsed
        .query_pairs()
        .find(|(k, _)| k == "access_token")
        .map(|(_, v)| v.to_string())
        .ok_or("token not found in that url")?;

    println!("Token successfully received");
    Ok(token)
}

pub async fn validate_token(token: &str) -> bool {
    let url = format!(
        "https://api.vk.com/method/users.get?access_token={token}&v=5.199"
    );
    let client = reqwest::Client::new();
    match client.get(url).send().await {
        Ok(resp) => {
            let text = resp.text().await.unwrap_or_default();
            !text.contains("\"error\"")
        }
        Err(_) => false,
    }
}
