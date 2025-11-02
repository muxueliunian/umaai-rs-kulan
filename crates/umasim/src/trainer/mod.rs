use anyhow::Result;
use log::info;
use rand::{Rng, prelude::StdRng};

use crate::game::{Game, Trainer, basic::BasicGame};

/// 猴子训练师
pub struct RandomTrainer;

impl<G: Game> Trainer<G> for RandomTrainer {
    fn select_action(&self, _game: &G, actions: &[<G as Game>::Action], rng: &mut StdRng) -> Result<usize> {
        let ret = rng.random_range(0..actions.len());
        info!("当前动作: {:?}, 随机选择 {:?}", actions, actions[ret]);
        Ok(ret)
    }
}
