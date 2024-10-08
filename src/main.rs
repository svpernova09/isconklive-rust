use std::error::Error;
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde_json::json;
use serde_json::Value;

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut alert_sent = false;
    log::info!("Initializing: alert_sent: {}", alert_sent);
    loop {
        match get_user_count() {
            Ok((user_count, room_status)) => {
                log::info!("User count: {}, Room status: {}", user_count, room_status);
                if room_status == 2 {
                    if !alert_sent {
                        if let Err(e) = send_discord_alert(user_count) {
                            log::error!("Failed to send discord alert: {}", e);
                        }
                        alert_sent = true;
                        log::info!("Discord Webhook sent. alert_sent: {}", alert_sent);
                    } else {
                        alert_sent = false;
                        log::info!(
                            "user_count: {}, alert_sent: {}",
                            user_count,
                            alert_sent
                        );
                    }
                } else {
                    alert_sent = false;
                    log::info!("Room Status wasn't 2/live?: room_status: {:?}", room_status);
                }
            }
            Err(e) => {
                log::error!("Failed to get user count: {}", e);
            }
        }

        // Sleep for 3 minutes
        thread::sleep(Duration::from_secs(4 * 60));
    }
}
fn get_user_count() -> Result<(u64, u64), Box<dyn Error>> {
    let client = Client::new();
    let response = client
        .get("https://www.tiktok.com/@conkdetects/live")
        .send()?;
    if response.status().is_success() {
        // Process the successful response
        let body = response.text()?;
        let document = Html::parse_document(&body);
        let script_selector = Selector::parse(r#"script[id="SIGI_STATE"]"#).unwrap();
        // Rust crashes here if `document.select(&script_selector).next()` returns None
        // check if document.select(&script_selector).next() is None
        if document.select(&script_selector).next().is_none() {
            log::warn!("Script element not found");
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Script element not found",
            )));
        } else {
            let script_element = document.select(&script_selector).next().unwrap();
            let json_str = script_element.text().collect::<String>();
            let parsed: Value = serde_json::from_str(&json_str)?;
            let room_status = parsed["LiveRoom"]["liveRoomUserInfo"]["liveRoom"]["status"]
                .as_u64()
                .unwrap_or(0);
            let user_count = parsed["LiveRoom"]["liveRoomUserInfo"]["liveRoom"]["liveRoomStats"]
                ["userCount"]
                .as_u64()
                .unwrap_or(0);
            Ok((user_count, room_status))
        }
    } else if response.status().is_client_error() {
        // Handle client error (4xx)
        log::warn!("400 from Server: Response: {:?}", response);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Client error occurred",
        )))
    } else if response.status().is_server_error() {
        // Handle server error (5xx)
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Server error occurred",
        )))
    } else {
        log::warn!("Something went really wrong: {:?}", response);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unknown error occurred",
        )))
    }
}

fn send_discord_alert(user_count: u64) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let webhook_url = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL must be set");
    let payload = json!({
        "content": format!("ConkDetects is live with {} viewers: https://www.tiktok.com/@conkdetects/live", user_count),
    });
    let response = client.post(webhook_url).json(&payload).send()?;
    log::info!("Discord Response: {:?}", response);
    Ok(())
}
