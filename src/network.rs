use std::time::Duration;

use reqwest::blocking::Client;

pub fn fetch(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .user_agent("RIFT-CLI/0.1.0")
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .map_err(|e| format!("Client initialization error: {e}"))?;

    let response = client.get(url).send().map_err(|e| {
        if e.is_timeout() {
            "Request timed out (the server took too long to respond)".to_string()
        } else if e.is_connect() {
            "Failed to connect (host unreachable, server down, or invalid domain)".to_string()
        } else {
            e.to_string()
        }
    })?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "HTTP {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Error")
        ));
    }

    response
        .text()
        .map_err(|e| format!("Failed to read response body: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_invalid_url() {
        let result = fetch("invalid://not-a-domain");
        assert!(result.is_err());
    }
}

