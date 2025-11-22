use anyhow::Result;
use log::info;
use rand::{Rng, prelude::StdRng, seq::SliceRandom};

use crate::{
    game::{ActionEnum, BaseAction, Game, Trainer, basic::BasicGame},
    gamedata::ActionValue
};

/// 猴子训练师
pub struct RandomTrainer;

impl<G: Game> Trainer<G> for RandomTrainer {
    fn select_action(&self, game: &G, actions: &[<G as Game>::Action], rng: &mut StdRng) -> Result<usize> {
        let mut random_index: Vec<_> = (0..actions.len()).collect();
        let mut ret = 0;
        random_index.shuffle(rng);
        for i in random_index {
            // 优先休息，会心情，训练。都不满足就随机第一个
            if game.uma().vital < 45 {
                if actions[i].as_base_action() == Some(BaseAction::Sleep) {
                    ret = i;
                    break;
                }
            } else if game.uma().motivation < 5 {
                if matches!(
                    actions[i].as_base_action(),
                    Some(BaseAction::NormalOuting) | Some(BaseAction::FriendOuting)
                ) {
                    ret = i;
                    break;
                }
            } else {
                if matches!(actions[i].as_base_action(), Some(BaseAction::Train(_))) {
                    ret = i;
                    break;
                }
            }
        }
        info!("吗喽训练员选择：{:?}", actions[ret]);
        Ok(ret)
    }

    fn select_choice(&self, game: &G, choices: &[ActionValue], rng: &mut StdRng) -> Result<usize> {
        let ret = rng.random_range(0..choices.len());
        info!("当前选项: {:?}, 随机选择选项 {}", choices, ret + 1);
        Ok(ret)
    }
}
