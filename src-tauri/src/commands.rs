use std::{fs, path::PathBuf, borrow::BorrowMut, thread::{spawn, Thread, self}};

use crossbeam_channel::unbounded;
use image::{DynamicImage, ImageBuffer, Rgba, GenericImageView};
use protobuf::MessageField;
use serde_json::Value;
use tauri::State;
use crate::{storage::{self, Profile}, util::{self, text_to_image}, rust_plus::{listen::RustPlus, protos::{self, rustplus::{AppEmpty, AppMessage}}}};
use anyhow::Result;

#[tauri::command]
pub fn connect_to_server(id: &str, servers: State<'_, storage::RustPlusServers>, runtime_info: State<'_, storage::RuntimeInformation>) {
    // Get a memory lock on the servers
    let mut connected_servers = servers.server.lock().unwrap();
    
    // If the server is already connected, don't connect again
    if connected_servers.contains_key(id) { return }

    // Find the data dir
    if let Some(data_dir) = runtime_info.data_dir.clone() {

        // Secure a memory lock on runtime information
        if let Profile::Open(profile_id) = runtime_info.profile.lock().unwrap().clone() {
            // Get the relevent paths
            let _profile_path =  data_dir.join(format!("profiles/{profile_id}"));
            let server_path =  data_dir.join(format!("profiles/{profile_id}/servers/{id}/"));

            println!("{:?}",server_path);
            
            // Open pair.json
            let pair_json = server_path.join("pair.json"); // Get the path
            let pair_json = fs::read_to_string(pair_json).unwrap(); // Read & Open the file
            let pair_json: Value = serde_json::from_str(&pair_json).unwrap(); // Parse and read as JSON

            // Decode pair.json
            let server_ip = pair_json["ip"].as_str().unwrap();
            let port = pair_json["port"].as_str().unwrap();
            let player_id = pair_json["playerId"].as_str().unwrap();
            let player_token = pair_json["playerToken"].as_str().unwrap();

            // Begin the connection
            let mut server = RustPlus::new(server_ip, port, player_id, player_token, false); // Setup the connection
            let (tx, rx) = unbounded(); // Create the message listener
            server.connect(tx).unwrap(); // Connect to the server

            // Cleanup
            let value = (server_path, server, rx); // Store the variables (rest will be dropped)
            connected_servers.insert((&id).to_string(), value); //

        }
    }
}

#[tauri::command]
pub fn get_connected_servers(servers: State<'_, storage::RustPlusServers>) {
    let servers = servers.server.lock().unwrap();
    println!("{:?}",servers);
}

/// Will get every map for a connected server!
#[tauri::command]
pub fn refresh_map(id: &str, servers: State<'_, storage::RustPlusServers>) {

    let offset = 0;

    // Get the server from memory
    let mut servers = servers.server.lock().unwrap(); // Get a lock
    let mut server: &mut (PathBuf, RustPlus, crossbeam_channel::Receiver<AppMessage>) = &mut servers.remove(id).unwrap(); // Remove the server from the hashmap
    
    // Deconstruct the tuple
    let dir = server.0.clone();
    let mut connected_server = server.1.borrow_mut();
    let rx = server.2.clone();

    // Craft the message
    let msg = protos::rustplus::AppRequest{
        getMap: MessageField::from(Some(AppEmpty::default())),
        ..Default::default()
    };

    let _ = &mut connected_server.send_message(msg);
    
    let map_response = rx.recv().unwrap();

    let msg = protos::rustplus::AppRequest{
        getInfo: MessageField::from(Some(AppEmpty::default())),
        ..Default::default()
    };

    let _ = &mut connected_server.send_message(msg);

    servers.insert(id.to_owned(), server.to_owned());

    let thread = thread::Builder::new().name("Map Generator".to_owned());

    thread.spawn(move || {
        let server_info_response = rx.recv().unwrap();

        let monuments = &map_response.response.map.monuments;
        let ingame_map_size = server_info_response.response.info.mapSize() as f32;
        
        let img = map_response.response.map.jpgImage().to_owned();
        let img_with_monuments = img.clone();

        let img_path = dir.join("map.jpg");
        spawn(move || {
            println!("Writing Blank Map");
            fs::write(img_path, img).unwrap();
            println!("Blank Map Written");
        });

        let ocean_margin = map_response.response.map.oceanMargin() as f32;
    
        let mut monument_img: DynamicImage = image::load_from_memory(&img_with_monuments).unwrap();
        let map_img_height = monument_img.height() as f32; // -2.0 * ocean_margin;
        let map_img_width = monument_img.width() as f32; // -2.0 * ocean_margin;
        for monument in monuments {
            let text = monument.clone().take_token();
            let (x, y) = (monument.clone().x(), monument.clone().y());
    
            let text_image = text_to_image(&text);
            // text_image.save(dir.join(format!("{text}.jpg"))).unwrap();
    
            // TODO: Get map size
            let scaled_x = x * ((map_img_width - 2.0 * ocean_margin) / ingame_map_size) + ocean_margin;
    
            let n = map_img_height - 2.0 * ocean_margin;
            let scaled_y = map_img_height - (y * (n / ingame_map_size) + ocean_margin);
    
            let x = scaled_x + offset as f32;
            let y = scaled_y + offset as f32;
    
            println!("Placing {text} at {x}, {y}");
        
            image::imageops::overlay(&mut monument_img, &text_image, x as i64, y as i64);
        }
        
        println!("Writing Named Map");
        monument_img.save(dir.join("map_with_monuments.jpg")).unwrap();
        println!("Named Map written");
            
    }).unwrap();
}

#[tauri::command]
pub fn get_servers(runtime_info: State<storage::RuntimeInformation>) -> Vec<String> {
    let mut servers = Vec::new();

    if let Some(data_dir) = runtime_info.data_dir.clone() {
        if let Profile::Open(id) = runtime_info.profile.lock().unwrap().clone() {
            servers.push("OK".to_owned());

            let data_dir = data_dir.join(format!("profiles/{id}/servers/"));
            println!("{:?}", data_dir);
            let mut found_servers = util::get_servers(data_dir).unwrap();
            servers.append(&mut found_servers);



        } else { servers.push("Please Select a Profile!".to_owned()) }

    } else { servers.push("Cannot Find Data Directory!".to_owned()) }

    servers
}

#[tauri::command]
pub fn set_profile(id: &str, runtime_info: State<'_, storage::RuntimeInformation>) {
    *runtime_info.profile.lock().unwrap() = Profile::Open(id.to_owned());
}