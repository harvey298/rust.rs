use std::{time, sync::Arc, borrow::BorrowMut};

use anyhow::Result;

use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};

// Define the payload format for the FCM listen API
#[derive(Debug, Serialize)]
struct ListenPayload {
    to: String,
    content_available: bool,
    priority: &'static str,
    registration_ids: Vec<String>,
}

// Define a function to listen for FCM notifications
async fn listen_for_fcm_notifications(
    fcm_credentials: &str,
) -> Result<Response, Error> {
    // Define the API endpoint and payload for listening to FCM notifications
    // let endpoint = "https://fcm.googleapis.com/fcm/listen";
    let endpoint = "https://mtalk.google.com";
    let payload = ListenPayload {
        to: fcm_credentials.to_string(),
        content_available: true,
        priority: "high",
        registration_ids: Vec::new(),
    };

    // Make an HTTP request to the FCM listen API
    let client = Client::new();
    let response = client
        .post(endpoint)
        .bearer_auth("976529667804")
        .json(&payload)
        .send()
        .await?;

    Ok(response)
}

// Define a function to handle incoming FCM notifications
async fn handle_fcm_notification<F>(notification: serde_json::Value, mut callback: F)
    where F: FnMut(&str)
{
    // Parse the notification body
    let placeholder = serde_json::json!({});
    let body = notification.get("data").unwrap_or(&placeholder);
    let body_str = serde_json::to_string_pretty(body).unwrap_or_else(|_| "".to_string());

    // Generate a timestamp for the notification
    // let timestamp = chrono::Local::now().to_rfc2822();
    let timestamp = 0;

    // Log the timestamp and the notification body to the console
    println!(
        "\x1b[32m[{}]\x1b[0m Notification Received: {}",
        timestamp, body_str
    );


    callback(&body_str);
}

/// Start listen to the FCM server
pub async fn listen<F>(fcm_credentials: &str, mut callback: F) -> Result<()> 
    where F: FnMut(&str)
{

    // Listen for incoming FCM notifications
    let response = listen_for_fcm_notifications(fcm_credentials).await?;

    println!("{:?}",response);

    let response = response.text().await.ok();
    

    // Parse the response as a JSON object and extract the notification data
    while let Some(line) = response.clone() {
        let json: serde_json::Value = serde_json::from_str(&line)?;
        if let Some(notification) = json.get("data") {
            // Handle the incoming notification
            handle_fcm_notification(notification.clone(), &mut callback).await;
        }
    }

    Ok(())
}


// #[test]
// pub fn fcm_test() {
//     #[tokio::main]
//     async fn handler() -> Result<()> {

//         // Create a new FirebaseMessaging instance with your FCM server key
//         let messaging = FirebaseMessaging::new("".to_owned())?;
        
//         // Start listening for incoming push notifications
//         messaging.start(|notification: NotificationMessage| {
//             // Handle incoming push notification here
//             println!("Received notification: {:?}", notification);
//         });

//         Ok(())
//     }

//     handler();
// }