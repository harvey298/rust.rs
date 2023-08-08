use std::{collections::HashMap, sync::{Arc, Mutex}, thread::spawn, mem};

use anyhow::Result;
use crossbeam_channel::{Sender, Receiver, unbounded};
use tungstenite::{connect, Message};
use protobuf::{CodedInputStream, Message as ProtoMessage, MessageField, CodedOutputStream};

use crate::{rust_plus::protos::{self, rustplus::AppEmpty}, config};

use super::protos::rustplus::{AppMessage, AppRequest};

pub type Callback = Arc<Box<dyn FnOnce(AppMessage)>>;

#[derive(Debug, Clone)]
pub struct RustPlus {
    server_ip: String,
    port: String,
    /// The Steam ID of the player
    player_id: String,
    /// From Server Pairing
    player_token: String,
    facepuch_proxy: bool,

    seq: u32,
    // seq_callbacks: HashMap<u32, Callback>,

    /// In some conditions this can be used to send a message to the server
    /// Sending messages this way will require you to pass the message through `init_message`
    pub internal_msg_sender: Sender<AppRequest>,
    internal_msg_receiver: Receiver<AppRequest>,
}

impl RustPlus {
    pub fn new(
        server_ip: &str,
        port: &str,
        player_id: &str,
        player_token: &str,
        facepuch_proxy: bool,

    ) -> Self {

        let (tx, rx) = unbounded();

        Self { 
            server_ip: server_ip.to_owned(),
            port: port.to_owned(),
            player_id: player_id.to_owned(),
            player_token: player_token.to_owned(),
            facepuch_proxy: facepuch_proxy,
            
            seq:0,
            // seq_callbacks: HashMap::new(),

            internal_msg_sender: tx,
            internal_msg_receiver: rx
        
        }

    }

    pub fn connect(&mut self, sender: Sender<AppMessage>)  -> Result<()> {
        let server = format!("{}:{}",self.server_ip, self.port);
        let url = if self.facepuch_proxy { format!("wss://companion-rust.facepunch.com/game/{}",server.replace(":", "/")) } else { format!("ws://{server}") };

        // Connect to the WebSocket server
        let (mut socket, _) = connect(url).expect("Failed to connect");


        let rx = self.clone().internal_msg_receiver.clone();

        spawn(move || {
            loop {
                println!("Awaiting Message to send");
        
                if let Ok(msg) = rx.recv() {
                    // let callback = msg.1;
                    // let msg = msg.0;
    
                    println!("Sending a message!");
                    if let Ok(msg) = msg.write_to_bytes() {
                        socket.write_message(tungstenite::Message::Binary(msg)).unwrap();
                    
                    }
                    
                }
    
                println!("Waiting for response");
                
                // Read the next WebSocket message
                let message = socket.read_message().expect("Failed to read message");
    
                match message {
                    Message::Binary(data) => {
                        // Parse the received message as a Protocol Buffer message
                        let mut input = CodedInputStream::from_bytes(&data);
                        let parsed_message = protos::rustplus::AppMessage::parse_from(&mut input)
                            .expect("Failed to parse message");
    
                        // Handle the received message
                        println!("Received message: {:?}", parsed_message);
    
                        // let seq = parsed_message.response.seq.unwrap();
                        sender.send(parsed_message).unwrap();
                    }
                    _ => {
                        println!("Received non-binary message");
                    }
                }
            }
        });
        

        Ok(())
    }

    pub fn send_message(&mut self, message: AppRequest) -> Result<()> {
        self.seq += 1;

        let mut message = message;
        message.seq = Some(self.seq);
        message.playerId = Some(self.player_id.parse().unwrap());
        message.playerToken = Some(self.player_token.parse().unwrap());

        let message = message;

        self.internal_msg_sender.send(message).unwrap();

        Ok(())
    }

    pub fn init_message(&self, message: AppRequest) -> Result<AppRequest> {
        let mut message = message;
        message.seq = Some(self.seq);
        message.playerId = Some(self.player_id.parse().unwrap());
        message.playerToken = Some(self.player_token.parse().unwrap());

        let message = message;

        Ok(message)
    }
}

#[test]
pub fn connect_test() {
    let cfg = config::Config::load("../config.toml").unwrap();
    let player_token = cfg.steam_token;

    let mut rs = RustPlus::new("45.88.230.60", "28039", "76561198314883513", "-1600858051", false);

    let (tx, rx) = unbounded();

    let mut rs2 = rs.clone();
    spawn(move || {
        rs2.connect(tx).unwrap();
    });

    let msg = protos::rustplus::AppRequest{
        getClanInfo: MessageField::from(Some(AppEmpty::default())),
        ..Default::default()
    };

    rs.send_message(msg).unwrap();

    loop {
        if let Ok(msg) = rx.recv() {
            println!("{:?}",msg);
        }

    }
}
