use anyhow::Result;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use crate::game::{base::*, *};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BaseAction {
    /// 训练
    Train(i32),
    /// 比赛
    Race,
    /// 休息
    Sleep,
    /// 友人出行
    FriendOuting,
    /// 普通出行
    NormalOuting,
    /// 治病
    Clinic
}

// 具体的操作是associate function, 都是 &Game -> Result<Game>的映射，产生一个新Game对象
// 其他Game的Action类型也可以直接调用BaseAction::do_train之类的方法得到BaseGame，然后再把结果转为自己的Game类型
impl BaseAction {
    pub fn do_train(game: &mut BaseGame, train: i32, rng: &mut StdRng) -> Result<()> {
        info!(">> 训练{train}");
        Ok(())
    }

    pub fn do_race(game: &mut BaseGame, rng: &mut StdRng) -> Result<()> {
        info!(">> 比赛");
        Ok(())
    }

    pub fn do_sleep(game: &mut BaseGame, rng: &mut StdRng) -> Result<()> {
        info!(">> 休息");
        Ok(())
    }

    pub fn do_friend_outing(game: &mut BaseGame, rng: &mut StdRng) -> Result<()> {
        info!(">> 友人出行");
        Ok(())
    }

    pub fn do_normal_outing(game: &mut BaseGame, rng: &mut StdRng) -> Result<()> {
        info!(">> 普通出行");
        Ok(())
    }

    pub fn do_clinic(game: &mut BaseGame, rng: &mut StdRng) -> Result<()> {
        info!(">> 治病");
        Ok(())
    }
}

/// 实现基础动作对基础游戏状态的变换，作为实际剧本动作的一部分
impl ActionEnum for BaseAction {
    type Game = BaseGame;
    fn apply(&self, game: &mut Self::Game, rng: &mut StdRng) -> Result<()> {
        match self {
            BaseAction::Train(train) => BaseAction::do_train(game, *train, rng),
            BaseAction::Race => BaseAction::do_race(game, rng),
            BaseAction::Sleep => BaseAction::do_sleep(game, rng),
            BaseAction::FriendOuting => BaseAction::do_friend_outing(game, rng),
            BaseAction::NormalOuting => BaseAction::do_normal_outing(game, rng),
            BaseAction::Clinic => BaseAction::do_clinic(game, rng)
        }
    }
}
