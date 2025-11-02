//! umaai-rs - Rewrite UmaAI in Rust
//!
//! author: curran
use anyhow::Result;

use crate::{gamedata::init_global, utils::init_logger};

pub mod explain;
pub mod game;
pub mod gamedata;
pub mod trainer;
pub mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    init_global()?;

    Ok(())
}
