//! Used for Internal global storage of things such as but not limited to;
//! 
//! Rust server connections
//! User details
//! 

use std::{sync::Mutex, collections::HashMap, path::PathBuf};

use crossbeam_channel::Receiver;
use tauri::Config;

use crate::rust_plus::{listen::RustPlus, protos::rustplus::AppMessage};

#[derive(Debug)]
pub struct RustPlusServers {
    pub server: Mutex<HashMap<String, (PathBuf, RustPlus, Receiver<AppMessage>)>>
}

impl Default for RustPlusServers {
    fn default() -> Self {
        Self { server: Default::default() }
    }
}

#[derive(Debug)]
pub struct RuntimeInformation {
    // pub cfg: Config,
    pub data_dir: Option<PathBuf>,
    pub profile: Mutex<Profile>
}

#[derive(Debug, Clone)]
pub enum Profile {
    Open(String),
    Closed,
}

impl Default for Profile {
    fn default() -> Self {
        Self::Closed
    }
}