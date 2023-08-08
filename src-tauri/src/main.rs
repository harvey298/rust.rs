// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{thread::{spawn, sleep}, time::Duration, path::Path, rc::Rc, sync::{Arc, Mutex}, fs};

use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
use tauri::{Window, api::process::{Command, CommandEvent}, utils::assets::EmbeddedAssets, Context, Menu, State};

mod http_server;
mod rust_plus;
mod config;
mod storage;
mod commands;
mod util;

lazy_static!(
    pub static ref CONFIG_PATH: &'static str = {
        "../config.toml"
    };

    pub static ref FCM_CREDS_STORE_PATH: &'static str = {
        "../creds.json"
    };

    pub static ref PAIR_NOTIF_KILLER: (Sender<()>, Receiver<()>) = {
        unbounded::<()>()
    };

);

#[tauri::command]
async fn listen_for_pair_notifcations() {

    let (mut rx_listener, mut child) = Command::new_sidecar("pair_listener")
    .expect("failed to create `pair_listener` binary command")
    .args(&["listen",*FCM_CREDS_STORE_PATH])
    .spawn()
    .expect("Failed to spawn sidecar");

    tokio::spawn(async move { loop {
        println!("Waiting for reponse");
        while let Some(event) = rx_listener.recv().await {
            if let CommandEvent::Stdout(line) = event {

                println!("{line}");

            }
        }
    }});

    tokio::spawn(async move {{
        PAIR_NOTIF_KILLER.1.recv().unwrap();
        println!("Killer Activated");
        child.kill().unwrap();
    }});

}

#[tauri::command]
async fn steam_login(handle: tauri::AppHandle) {
    // check if I'm already logged in
    if Path::new(*CONFIG_PATH).exists() { return }

    let steam_login_window = tauri::WindowBuilder::new(
        &handle,
        "steam_login", /* the unique window label */
        tauri::WindowUrl::External("https://companion-rust.facepunch.com/login".parse().unwrap())
    ).title("Rust+ Login").build().unwrap();


    #[cfg(debug_assertions)]
    steam_login_window.open_devtools();

    let js = "
    // Borrowed from https://github.com/liamcottle/rustplus.js
    console.log('Code injected!');

    if(window.ReactNativeWebView === undefined){
        var handlerInterval = setInterval(function() {

            if(window.ReactNativeWebView === undefined){
                        
                console.log('registering ReactNativeWebView.postMessage handler');
                window.ReactNativeWebView = {

                    /**
                     * Rust+ login website calls ReactNativeWebView.postMessage after successfully logging in with Steam.
                     * @param message json string with SteamId and Token
                     */
                    postMessage: function(message) {

                        console.log('Token recived');

                        // we no longer need the handler
                        clearInterval(handlerInterval);

                        // parse json message
                        var auth = JSON.parse(message);

                        // Send the token to the Rust backend
                        var token = auth.Token;

                        // Send the Steam token to the server
                        fetch(`http://127.0.0.1:4352/login?token=${token}`)
                        .then(response => response.text())
                        .then(data => console.log(data))
                        .catch(error => console.error(error));

                    },

                };
            }
        }, 250);
    }";

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let (tx, rx) = unbounded::<String>();

    let (mut rx_listener, mut child) = Command::new_sidecar("pair_listener")
    .expect("failed to create `pair_listener` binary command")
    .args(&["reg",*FCM_CREDS_STORE_PATH])
    .spawn()
    .expect("Failed to spawn sidecar");

    spawn(move || { println!("Starting internal HTTP server");http_server::server(tx.clone(), shutdown_rx).unwrap();println!("Internal HTTP server has fully ended") });    
    
    let window_2 = steam_login_window.clone();
    spawn(move || loop {
        sleep(Duration::from_secs(1));
        window_2.eval(js).unwrap();
    });

    let window_2 = steam_login_window.clone();
    tokio::spawn(async move { loop {
        if let Ok(token) = rx.try_recv() {
            println!("Main Thread has token!");
            shutdown_tx.send(()).unwrap();
            window_2.close().unwrap();

            let event = rx_listener.recv().await.unwrap();

            if let CommandEvent::Stdout(line) = event {
                println!("{line}");
                
                let fcm_key = line.replace("ID:","");
            
                let expo_token = rust_plus::get_expo_push_token(&fcm_key).await.unwrap();

                let token = token.replace("token=", "");
    
                rust_plus::register_with_rust_plus(&token, &expo_token).await.unwrap();

                let cfg = config::Config{fcm_credentials: fcm_key.to_owned(), expo_token: expo_token, steam_token: token};
                cfg.save(*CONFIG_PATH).unwrap();
                println!("I have registered with Rust+");
            }
                
            break
        }

        // Window Check
        if let Err(_) = window_2.is_visible() {            
            shutdown_tx.send(()).unwrap();
            break
        }
    }});

}

fn main() {
    
    let context = tauri::generate_context!();
    let config = context.config();

    let cfg = config.clone();

    let mut runtime_info = storage::RuntimeInformation{ 
        // cfg,
        data_dir: None,
        profile: Mutex::new(storage::Profile::Closed),
    };

    // Create the data dir
    if let Some(path) = tauri::api::path::app_data_dir(config) {
        if !Path::new(&path).exists() {
            fs::create_dir_all(path.clone()).unwrap();
        }

        runtime_info.data_dir = Some(path);
    }

    let cleanup_handler = || {
        println!("Program is exiting!");

    };

    let menu = Menu::new();

    tauri::Builder::default()
        .menu(menu)
        .on_window_event(move |event| {

            match event.event() {
                // tauri::WindowEvent::Resized(_) => todo!(),
                // tauri::WindowEvent::Moved(_) => todo!(),
                // tauri::WindowEvent::CloseRequested { api , .. } => todo!(),
                tauri::WindowEvent::Destroyed => { cleanup_handler(); println!("Closing Backend"); },
                // tauri::WindowEvent::Focused(_) => todo!(),
                // tauri::WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size , .. } => todo!(),
                // tauri::WindowEvent::FileDrop(_) => todo!(),
                // tauri::WindowEvent::ThemeChanged(_) => todo!(),
                _ => {},
            }

        })
        // Setup storage
        .manage(storage::RustPlusServers::default())
        .manage(runtime_info)

        // Being building the app
        .invoke_handler(tauri::generate_handler![

            steam_login,
            listen_for_pair_notifcations,
            commands::get_servers,
            commands::set_profile,
            commands::connect_to_server,
            commands::get_connected_servers,
            commands::refresh_map
            
        ])
        .run(context)
        .expect("error while running tauri application");
}
