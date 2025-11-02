//! 基础（无剧本）游戏，用于测试特性

use std::ops::{Deref, DerefMut};

use anyhow::Result;
use colored::Colorize;
use comfy_table::{ColumnConstraint, ContentArrangement, Table, Width};
use enum_iterator::Sequence;
use hashbrown::HashMap;
use log::info;
use rand::rngs::StdRng;

use crate::{
    game::{
        BaseAction::{self, *},
        BaseGame,
        BasePerson,
        FriendOutState,
        InheritInfo,
        SupportCard,
        TurnStage,
        Uma,
        traits::*
    },
    gamedata::*
};

#[derive(Debug, Clone, PartialEq)]
pub struct BasicAction(BaseAction);

impl Deref for BasicAction {
    type Target = BaseAction;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ActionEnum for BasicAction {
    type Game = BasicGame;
    fn apply(&self, game: &mut Self::Game, rng: &mut StdRng) -> anyhow::Result<()> {
        self.0.apply(game, rng)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BasicGame {
    pub base: BaseGame,
    pub persons: Vec<BasePerson>
}

impl Deref for BasicGame {
    type Target = BaseGame;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BasicGame {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl BasicGame {
    pub fn is_xiahesu(&self) -> bool {
        (self.turn >= 36 && self.turn < 40) || (self.turn >= 60 && self.turn < 64)
    }

    pub fn add_person(&mut self, mut person: BasePerson) {
        info!("新训练角色: {}", person.explain());
        person.person_index = self.persons.len() as i32;
        self.persons.push(person);
    }

    pub fn is_race_turn(&self) -> Result<bool> {
        self.uma.is_race_turn(self.turn as u32)
    }

    pub fn newgame(uma_id: u32, deck_ids: &[u32; 6], inherit: InheritInfo) -> Result<Self> {
        let mut ret = BasicGame {
            base: BaseGame::new(uma_id, deck_ids, inherit)?,
            persons: vec![]
        };
        ret.init_persons()?;
        Ok(ret)
    }
}

impl Game for BasicGame {
    type Person = BasePerson;
    type Action = BasicAction;
    fn init_persons(&mut self) -> Result<()> {
        let persons = self
            .deck
            .iter()
            .map(|card| BasePerson::try_from(card))
            .collect::<Result<Vec<_>>>()?;
        for p in persons {
            self.add_person(p);
        }
        // 添加理事长
        self.add_person(BasePerson::yayoi());
        Ok(())
    }
    fn next(&mut self) -> bool {
        if let Some(stage) = self.stage.next() {
            // 回合内，下一个阶段
            self.stage = stage;
        } else if self.turn < self.max_turn() {
            // 下一个回合
            self.turn += 1;
            self.stage = TurnStage::Begin;
        } else {
            return false;
        }
        true
    }

    fn list_actions(&self) -> Result<Vec<Self::Action>> {
        let mut actions = vec![];
        if self.is_race_turn()? {
            Ok(vec![BasicAction(Race)])
        } else {
            actions = vec![
                BasicAction(Train(0)),
                BasicAction(Train(1)),
                BasicAction(Train(2)),
                BasicAction(Train(3)),
                BasicAction(Train(4)),
            ];
            if self.is_xiahesu() {
                actions.push(BasicAction(Race));
                actions.push(BasicAction(NormalOuting));
            } else {
                // 普通训练
                actions.push(BasicAction(Sleep));
                actions.push(BasicAction(NormalOuting));
                if self.turn > 13 && self.turn < 72 {
                    actions.push(BasicAction(Race));
                }
                if self.uma.flags.ill {
                    actions.push(BasicAction(Clinic));
                }
                if self.friend.out_state == FriendOutState::AfterUnlock && self.turn < 72 {
                    if !self.friend.out_used.iter().all(|used| *used) {
                        actions.push(BasicAction(FriendOuting));
                    }
                }
            }
            Ok(actions)
        }
    }

    fn list_events(&self) -> Vec<EventData> {
        vec![]
    }

    fn run_stage<T: Trainer<Self>>(&mut self, trainer: &T, rng: &mut StdRng) -> Result<()> {
        //let events = self.list_events();
        info!("-- Turn {}-{:?} --", self.turn, self.stage);
        match self.stage {
            TurnStage::Distribute => {
                if self.is_race_turn()? {
                    self.reset_distribution();
                } else {
                    self.distribute_all(rng)?;
                    info!("训练:\n{}", self.explain_distribution());
                }
            }
            TurnStage::Train => {
                let actions = self.list_actions()?;
                let selection = trainer.select_action(self, &actions, rng)?;
                //info!("玩家选择: {:?}", actions[selection]);
                self.apply_action(&actions[selection], rng)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn deyilv(&self, person_index: i32) -> Result<f32> {
        if person_index < 6 {
            let (eff, _) = self.deck[person_index as usize].calc_training_effect(self, 0)?;
            Ok(eff.deyilv)
        } else {
            Ok(0.0)
        }
    }
    fn explain_distribution(&self) -> String {
        let headers = vec!["速", "耐", "力", "根", "智"];
        let dist = &self.distribution;
        let mut rows = vec![];
        for i in 0..6 {
            let mut row = vec![];
            for train in 0..5 {
                if let Some(id) = dist[train].get(i) {
                    let text = self.persons[*id as usize].explain();
                    row.push(text);
                } else {
                    row.push("".to_string());
                }
            }
            rows.push(row)
        }
        let mut table = Table::new();
        table.set_header(headers).add_rows(rows).set_width(80);
        for col in table.column_iter_mut() {
            col.set_constraint(ColumnConstraint::Absolute(Width::Percentage(20)));
        }
        table.to_string()
    }
    // getters
    fn persons(&self) -> &[Self::Person] {
        &self.persons
    }
    fn absent_rate_drop(&self) -> i32 {
        self.absent_rate_drop
    }
    fn turn(&self) -> i32 {
        self.turn
    }
    fn max_turn(&self) -> i32 {
        77
    }
    fn uma(&self) -> &Uma {
        &self.uma
    }
    fn uma_mut(&mut self) -> &mut Uma {
        &mut self.uma
    }
    fn deck(&self) -> &Vec<SupportCard> {
        &self.deck
    }
    fn events(&self) -> &HashMap<u32, u32> {
        &self.events
    }
    fn event_mut(&mut self) -> &mut HashMap<u32, u32> {
        &mut self.events
    }
    fn distribution(&self) -> &Vec<Vec<i32>> {
        &self.distribution
    }
    fn distribution_mut(&mut self) -> &mut Vec<Vec<i32>> {
        &mut self.distribution
    }
    fn has_group_buff(&self) -> bool {
        self.friend.group_buff_turn > 0
    }
    fn train_level(&self, train: usize) -> usize {
        self.train_level_count[train] as usize / 4 + 1
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rand::SeedableRng;

    use super::*;
    use crate::{gamedata::*, global, trainer::RandomTrainer, utils::*};

    #[test]
    fn test_newgame() -> Result<()> {
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
        game.run_simulate(&trainer, &mut rng)?;
        Ok(())
    }
}
