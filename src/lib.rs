pub mod commands;
pub mod config;
pub mod devops;
pub mod error;
pub mod graph;
pub mod keyring;
pub mod pace;
pub mod platform;
pub mod state;
pub mod utils;

use clap::ValueEnum;
use serde::Serialize;

#[derive(Clone, Copy, ValueEnum, Debug, Default, Serialize)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}
