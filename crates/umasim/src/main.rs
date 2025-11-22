//! umaai-rs - Rewrite UmaAI in Rust
//!
//! author: curran
use anyhow::Result;
use log::info;
use rand::{SeedableRng, rngs::StdRng};

use crate::{
    game::{Game, InheritInfo, basic::BasicGame},
    gamedata::{GAMECONSTANTS, init_global},
    trainer::RandomTrainer,
    utils::init_logger
};

pub mod explain;
pub mod game;
pub mod gamedata;
pub mod trainer;
pub mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    init_global()?;
    let mut game = BasicGame::newgame(101901, &[302424, 302464, 302484, 302564, 302574, 302644], InheritInfo {
        blue_count: [15, 3, 0, 0, 0],
        extra_count: [0, 30, 0, 0, 30, 30]
    })?;
    println!("{}", game.explain()?);
    let score = game.uma.calc_score();
    println!("评分: {} {}", global!(GAMECONSTANTS).get_rank_name(score), score);
    let trainer = RandomTrainer {};
    let mut rng = StdRng::from_os_rng();
    game.run_full_game(&trainer, &mut rng)?;
    info!("育成结束！");
    let score = game.uma.calc_score();
    println!(
        "评分: {} {}, PT: {}",
        global!(GAMECONSTANTS).get_rank_name(score),
        score,
        game.uma.total_pt()
    );
    Ok(())
}
