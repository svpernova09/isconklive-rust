use std::error::Error;
use std::time::Duration;
use std::thread;

use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde_json::json;
use serde_json::Value;

fn main() -> Result<(), Box<dyn Error>> {
    let mut alert_sent = false;

    loop {
        let client = Client::new();
        let response = client
            .get("https://www.tiktok.com/@conkdetects/live")
            .send()?;
        let body = response.text()?;
        let document = Html::parse_document(&body);
        let script_selector = Selector::parse(r#"script[id="SIGI_STATE"]"#).unwrap();
        let script_element = document.select(&script_selector).next().unwrap();
        let json_str = script_element.text().collect::<String>();
        let parsed: Value = serde_json::from_str(&json_str)?;
        let user_count = parsed["LiveRoom"]["liveRoomUserInfo"]["liveRoom"]["liveRoomStats"]
            ["userCount"]
            .as_u64()
            .unwrap_or(0);

        println!("User count: {}", user_count);
        if user_count > 10 && !alert_sent {
            send_discord_alert(user_count)?;
            alert_sent = true;
            println!("Discord Webhook sent. alert_sent: {}", alert_sent);
        } else if user_count < 10 {
            alert_sent = false;
            println!("Usercount below 10. user_count: {}, alert_sent: {}", user_count, alert_sent);
        }

        // Sleep for 5 minutes
        thread::sleep(Duration::from_secs(5 * 60));
    }
}

fn send_discord_alert(user_count: u64) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let webhook_url = std::env::var("WEBHOOK_URL").expect("WEBHOOK_URL must be set");
    let payload = json!({
        "content": format!("ConkDetects is live with {} viewers: https://www.tiktok.com/@conkdetects/live", user_count),
    });
    let response = client.post(webhook_url).json(&payload).send()?;
    println!("Response: {:?}", response);
    Ok(())
}
