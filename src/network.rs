use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use reqwest::blocking::Client;

pub fn find_chromium() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("CHROMIUM_PATH") {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    let candidates = [
        "chromium",
        "chromium-browser",
        "google-chrome",
        "google-chrome-stable",
        "chrome",
        "brave-browser",
        "/snap/bin/chromium",
        "/usr/bin/chromium-browser",
        "/usr/bin/chromium",
        "/usr/bin/google-chrome",
    ];

    for candidate in candidates {
        if candidate.starts_with('/') {
            let p = PathBuf::from(candidate);
            if p.exists() {
                return Some(p);
            }
        } else if let Ok(output) = Command::new("which").arg(candidate).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let p = PathBuf::from(path_str);
                    if p.exists() {
                        return Some(p);
                    }
                }
            }
        }
    }

    None
}

pub fn is_chromium_available() -> bool {
    find_chromium().is_some()
}

pub fn fetch(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::builder()
        .user_agent("RIFT-CLI/0.1.0")
        .timeout(Duration::from_secs(15))
        .build()?;

    let response = client.get(url).send()?.error_for_status()?;

    Ok(response.text()?)
}

pub fn fetch_with_chromium(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let chromium_bin = find_chromium()
        .ok_or("Chromium not found. Install chromium/google-chrome or set CHROMIUM_PATH.")?;

    let output = Command::new(chromium_bin)
        .args([
            "--headless=new",
            "--no-sandbox",
            "--disable-gpu",
            "--virtual-time-budget=2000",
            "--dump-dom",
            url,
        ])
        .output()?;

    let html = String::from_utf8_lossy(&output.stdout).to_string();
    if html.trim().is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Chromium returned empty DOM. {stderr}").into());
    }

    Ok(html)
}

pub fn fetch_with_options(url: &str, use_js: bool) -> Result<String, Box<dyn std::error::Error>> {
    if use_js {
        fetch_with_chromium(url)
    } else {
        fetch(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_chromium_does_not_panic() {
        let _ = find_chromium();
    }

    #[test]
    fn test_is_chromium_available() {
        assert_eq!(is_chromium_available(), find_chromium().is_some());
    }

    #[test]
    fn test_fetch_invalid_url() {
        assert!(fetch("invalid-url").is_err());
    }

    #[test]
    #[ignore] // Live network test with Chromium: cargo test -- --ignored
    fn test_fetch_with_chromium_live() {
        if is_chromium_available() {
            let res = fetch_with_chromium("https://smitronix.dev");
            assert!(res.is_ok());
            let html = res.unwrap();
            assert!(html.contains("<html"));
        }
    }
}
