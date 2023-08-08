use std::collections::HashMap;
use hyper::header::{CONTENT_TYPE, CONTENT_LENGTH};
use reqwest::header::{AUTHORIZATION, HeaderValue, HeaderMap};
use serde_json::{json, Value};
use uuid::Uuid;

pub mod listen;
pub mod protos;
pub mod pair_listen;

pub const RUST_PLUS_DEFAULT_PORT: usize = 28082;

/// Gathers the expo push token for Rust+
pub async fn get_expo_push_token(credentials: &str) -> Result<String, reqwest::Error> {
    let uuid = Uuid::new_v4();
    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    // headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let data = json!({
        "deviceId": uuid.hyphenated().to_string(),
        "experienceId": "@facepunch/RustCompanion",
        "appId": "com.facepunch.rust.companion",
        "deviceToken": credentials,
        "type": "fcm",
        "development": false
    });
    
    let payload = serde_json::to_vec(&data).unwrap();
    headers.insert(CONTENT_LENGTH, HeaderValue::from_str(&(payload.len()).to_string()).unwrap());

    let response = client.post("https://exp.host/--/api/v2/push/getExpoPushToken")
        .headers(headers)
        .body(payload)
        .send().await?.text().await?;

    // println!("data: {}", response);
    let response: Value = serde_json::from_str(&response).unwrap(); 
    let token = &response["data"]["expoPushToken"];

    let expo_push_token = token.as_str().unwrap().to_owned();
    Ok(expo_push_token)
}

/// Uses a steam auth token and the expo push token gathered from `get_expo_push_token` to register the app with Rust+ servers
pub async fn register_with_rust_plus(steam_auth_token: &str, expo_push_token: &str) -> Result<(), reqwest::Error> {

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();

    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let data = json!({
        "AuthToken": steam_auth_token,
        "DeviceId": "rustplus.rs",
        "PushKind": 0,
        "PushToken": expo_push_token
    });
    
    let payload = serde_json::to_vec(&data).unwrap();
    headers.insert(CONTENT_LENGTH, HeaderValue::from_str(&(payload.len()).to_string()).unwrap());

    let response = client.post("https://companion-rust.facepunch.com:443/api/push/register")
        .headers(headers)
        .body(payload)
        .send().await?;

    println!("{}",response.text().await.unwrap());
    Ok(())
}

#[test]
pub fn pair_test() {

    #[tokio::main]
    pub async fn handler() {
        let token = get_expo_push_token("ffc0Mj179GQ:APA91bF9uD3jm1dK39qbBnnYSHd2IsaajWAdGkH6kiIMTBFzxl4zdgn8GAZSLad_QWqqIFK5aNYGKgmeMjnef4e796dfJsqSjeFOiSGvHFpbkHbcmmBlLYmkFtDnu2W3DxILFjrg84PT").await.unwrap();
        println!("{}",token);

	let token = token.replace("ExponentPushToken[","");
	let token = token.replace("]","");

	register_with_rust_plus("eyJzdGVhbUlkIjoiNzY1NjExOTgzMTQ4ODM1MTMiLCJ2ZXJzaW9uIjowLCJpc3MiOjE2ODIwMzAwMTMsImV4cCI6MTY4MzIzOTYxM30=.o89pHNPfv5ztfM11/bPhrB6w7uA2Vil7WKF2BckiknMXaXAaaztTHECIzs+Y4qXBDK9BfShAK4ItbV7RDhapDQ==", &token).await.unwrap();
    }

    handler()

}