use std::io::{self, Write};
use url::Url;

const VK_AUTH_URL: &str = "https://oauth.vk.com/authorize?client_id=6287487&display=page&\
                           redirect_uri=https://oauth.vk.com/blank.html&scope=messages,offline&\
                           response_type=token&v=5.199";

pub async fn get_access_token() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    println!("Opening browser for authorization...");

    // Attempt to open browser in background, but don't fail if it doesn't work
    std::thread::spawn(move || {
        let _ = open::that(VK_AUTH_URL);
    });

    print!("\nPaste the URL from your browser after authorization: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Err("No input provided".into());
    }

    let target_str = parse_auth_url(input);
    let parsed = Url::parse(&target_str)
        .map_err(|_| "Failed to parse URL")?;

    let token = parsed
        .query_pairs()
        .find(|(k, _)| k == "access_token")
        .map(|(_, v)| v.into_owned())
        .ok_or("Token not found in the provided URL")?;

    println!("Token successfully received!");
    Ok(token)
}

/// Normalizes various URL formats to extract the token from
fn parse_auth_url(input: &str) -> String {
    if input.contains('#') {
        // VK returns token in fragment: #access_token=...
        input.replace('#', "?")
    } else if input.contains("access_token=") {
        // Already has access_token parameter
        format!("http://localhost/?{}", input)
    } else {
        // Assume it's a complete URL
        input.to_string()
    }
}

pub async fn validate_token(token: &str) -> bool {
    let url = format!(
        "https://api.vk.com/method/users.get?access_token={}&v=5.199",
        token
    );
    
    match reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .and_then(|resp| resp.text())
    {
        Ok(text) => !text.contains("\"error\""),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_auth_url_with_fragment() {
        let input = "#access_token=test123&expires_in=0";
        assert_eq!(
            parse_auth_url(input),
            "?access_token=test123&expires_in=0"
        );
    }

    #[test]
    fn test_parse_auth_url_with_query() {
        let input = "access_token=test123&expires_in=0";
        assert_eq!(
            parse_auth_url(input),
            "http://localhost/?access_token=test123&expires_in=0"
        );
    }

    #[test]
    fn test_parse_auth_url_with_full_url() {
        let input = "http://example.com/?access_token=test123";
        assert_eq!(parse_auth_url(input), "http://example.com/?access_token=test123");
    }
}
