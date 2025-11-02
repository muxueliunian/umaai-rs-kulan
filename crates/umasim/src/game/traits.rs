use std::fmt::Debug;

use anyhow::{Result, anyhow};
use hashbrown::HashMap;
use log::{info, warn};
use rand::{Rng, rngs::StdRng};
use rand_distr::{Distribution, weighted::WeightedIndex};

use super::PersonType;
use crate::{
    game::{CardTrainingEffect, SupportCard, Uma},
    gamedata::{ActionValue, EventData, GAMECONSTANTS},
    global,
    utils::Array5
};
// Game为核心特性，
// ActionEnum 执行动作，修改Game状态
// Trainer 选择动作
// 对事件的处理由Game自己进行

/// 训练人头特性，用于随机分配
pub trait Person: Debug + Clone + PartialEq + Default {
    /// person type getter
    fn person_type(&self) -> PersonType;

    /// person index getter
    fn person_index(&self) -> i32;

    /// train type getter
    fn train_type(&self) -> i32;

    /// friendship getter
    fn friendship(&self) -> i32;

    /// provided: 是否为友人，团队，记者或者理事长
    fn is_friend(&self) -> bool {
        self.train_type() > 4 || matches!(self.person_type(), PersonType::Reporter | PersonType::Yayoi)
    }
}

/// 会改变Game状态的主动选项
pub trait ActionEnum: Debug {
    /// 操作的对象类型，不一定要实现Game Trait
    type Game;

    /// visitor，调用具体动作
    fn apply(&self, game: &mut Self::Game, rng: &mut StdRng) -> Result<()>;
}

/// 游戏状态类型需要实现的Trait，不包括初始化
pub trait Game: Clone {
    type Person: Person;
    type Action: ActionEnum<Game = Self>;

    // 回合相关
    fn turn(&self) -> i32;
    /// 最大回合数 getter
    fn max_turn(&self) -> i32;
    /// 下一阶段。如果已经结束，返回false
    fn next(&mut self) -> bool;
    /// 模拟当前Stage
    fn run_stage<T: Trainer<Self>>(&mut self, trainer: &T, rng: &mut StdRng) -> Result<()>;
    /// provided: 模拟到游戏结束
    fn run_simulate<T: Trainer<Self>>(&mut self, trainer: &T, rng: &mut StdRng) -> Result<()> {
        self.run_stage(trainer, rng)?;
        while self.next() {
            self.run_stage(trainer, rng)?;
        }
        Ok(())
    }
    // 动作相关
    /// events getter
    fn events(&self) -> &HashMap<u32, u32>;
    /// events mut
    fn event_mut(&mut self) -> &mut HashMap<u32, u32>;
    /// 获取当前可能的可控行动
    fn list_actions(&self) -> Result<Vec<Self::Action>>;
    /// 获取当前可能发动的事件
    fn list_events(&self) -> Vec<EventData>;
    /// provided: 执行指定事件
    fn apply_event(&mut self, event: &EventData, rng: &mut StdRng) -> Result<&mut Self> {
        let roll = rng.random_range(0..100);
        if event.trigger_prob >= 100 || roll < event.trigger_prob {
            info!(
                "+事件#{} {} [roll {}<{}]",
                event.id, event.name, roll, event.trigger_prob
            );
            // 记录触发
            self.event_mut().entry(event.id).and_modify(|x| *x += 1).or_insert(1);
            // 增加效果
            self.uma_mut().apply_action(&event.bonus);
        }
        Ok(self)
    }
    /// provided: 执行指定动作
    fn apply_action(&mut self, action: &Self::Action, rng: &mut StdRng) -> Result<()> {
        action.apply(self, rng)
    }
    // 人头分配相关
    /// persons getter
    fn persons(&self) -> &[Self::Person];
    /// 初始化人头
    fn init_persons(&mut self) -> Result<()>;
    /// distribution getter
    fn distribution(&self) -> &Vec<Vec<i32>>;
    /// distribution mut
    fn distribution_mut(&mut self) -> &mut Vec<Vec<i32>>;
    /// absent_rate_drop getter
    fn absent_rate_drop(&self) -> i32;
    /// 计算得意率
    fn deyilv(&self, person_index: i32) -> Result<f32>;
    /// 团队卡是否可以闪彩，不考虑多个团卡的情况
    fn has_group_buff(&self) -> bool;
    /// 显示分布信息
    fn explain_distribution(&self) -> String;
    /// 重置分布
    fn reset_distribution(&mut self) {
        self.distribution_mut().clear();
        for _ in 0..5 {
            self.distribution_mut().push(vec![]);
        }
    }
    /// 追加分配一个在persons里已经存在的人头, -1为不在
    /// 如果要新加角色 需要手动添加到persons里
    fn distribute_person(&mut self, person_index: i32, allow_absent: bool, rng: &mut StdRng) -> Result<i32> {
        let person = &self.persons()[person_index as usize];
        let train_type = person.train_type() as usize;
        // 计算不在率
        let mut absent_rate = match person.person_type() {
            PersonType::Card => 50 - self.absent_rate_drop(),
            PersonType::Yayoi | PersonType::Reporter => 200,
            _ => 100 - self.absent_rate_drop()
        };
        if !allow_absent {
            absent_rate = 0;
        }
        // 计算得意率权重
        let mut weights = vec![100, 100, 100, 100, 100, absent_rate];
        if train_type <= 4 {
            weights[train_type] += self.deyilv(person_index)? as i32;
        }
        let dist_absent = WeightedIndex::new(&weights)?;
        let dist = WeightedIndex::new(&weights[0..5])?;
        // 先判断是否不在
        if dist_absent.sample(rng) == 5 {
            Ok(-1)
        } else {
            // 尝试分配
            let d = self.distribution();
            let mut ok = false;
            let mut retries = 0;
            let mut train = 0;
            while !ok && retries < 10 {
                train = dist.sample(rng);
                retries += 1;
                // 不能多于5人或出现同样人头
                if d[train].len() >= 5 || d[train].contains(&person_index) {
                    continue;
                }
                // 每个训练只能出现一个友人
                if person.is_friend() && d[train].iter().any(|index| self.persons()[*index as usize].is_friend()) {
                    continue;
                }
                ok = true;
            }
            if !ok {
                warn!("分配角色#{person_index}失败");
                Ok(-1)
            } else {
                self.distribution_mut()[train as usize].push(person_index);
                Ok(train as i32)
            }
        }
    }
    /// 重新分配所有人头
    fn distribute_all(&mut self, rng: &mut StdRng) -> Result<()> {
        let sequence = vec![
            PersonType::Yayoi,
            PersonType::Reporter,
            PersonType::ScenarioCard,
            PersonType::TeamCard,
            PersonType::Card,
            PersonType::Npc,
        ];
        self.reset_distribution();
        for ty in sequence {
            for i in 0..self.persons().len() {
                if self.persons()[i].person_type() == ty {
                    self.distribute_person(i as i32, true, rng)?;
                }
            }
        }
        Ok(())
    }
    // provided: 指定人头出现在训练中的位置
    fn at_trains(&self, person_index: i32) -> Vec<bool> {
        self.distribution()
            .iter()
            .map(|train| train.contains(&person_index))
            .collect()
    }
    /// provided: 指定人头如果在指定位置是否会闪彩 train 0-4 速耐力根智 >=5暂时不考虑  
    /// 非默认实现需要依赖于一部分剧本Buff，所以要在Game里判断
    fn is_shining_at(&self, person_index: usize, train: usize) -> bool {
        let person = &self.persons()[person_index];
        match person.person_type() {
            PersonType::Card => person.train_type() == train as i32 && person.friendship() >= 80,
            PersonType::TeamCard => self.has_group_buff(),
            // 默认实现中其他卡不能闪彩
            _ => false
        }
    }
    /// provided: 指定训练的彩圈个数
    fn shining_count(&self, train: usize) -> usize {
        self.distribution()[train]
            .iter()
            .filter(|index| self.is_shining_at(**index as usize, train))
            .count()
    }
    // 训练相关
    /// 设施等级 getter
    fn train_level(&self, train: usize) -> usize;
    /// uma getter
    fn uma(&self) -> &Uma;
    /// uma mut getter
    fn uma_mut(&mut self) -> &mut Uma;
    /// deck getter
    fn deck(&self) -> &Vec<SupportCard>;
    /// provided: 计算来自支援卡的训练buff
    fn calc_training_buff(&self, train: usize) -> Result<CardTrainingEffect> {
        let mut ret = CardTrainingEffect::default();
        if train >= 5 {
            return Err(anyhow!("训练类型错误: {train}"));
        }
        for index in &self.distribution()[train] {
            if *index < 6 {
                let card = &self.deck()[*index as usize];
                let mut effect = card.effect.clone();
                if !self.is_shining_at(*index as usize, train) {
                    effect.youqing = 0.0;
                }
                ret = ret.add(&effect);
            }
        }
        Ok(ret)
    }
    /// provided: 计算训练属性
    fn calc_training_value(&self, buffs: &CardTrainingEffect, train: usize) -> Result<ActionValue> {
        let cons = global!(GAMECONSTANTS);
        let train_level = self.train_level(train);
        if train >= 5 {
            return Err(anyhow!("训练类型错误: {train}"));
        }
        // 人数, 排除掉理事长和记者
        let person_count = self.distribution()[train]
            .iter()
            .filter(|p| **p != 6 && **p != 7)
            .count();
        // 基础值
        let basic_value = &cons.training_basic_value[train][train_level];
        let basic_motivation = ((self.uma().motivation - 3) * 10) as f32;
        // 成长率
        let status_bonus = &self.uma().five_status_bonus;
        let mut ret = ActionValue::default();
        // 副属性
        for i in 0..6 {
            if basic_value[i] > 0 {
                ret.status_pt[i] = basic_value[i] + buffs.bonus[i];
            }
        }
        ret.vital = basic_value[6];
        // 直接计算。假设buffs里已经算好中间加成
        for i in 0..6 {
            if basic_value[i] > 0 {
                ret.status_pt[i] = (ret.status_pt[i] as f32
                    * (1.0 + 0.01 * buffs.youqing as f32)
                    * (1.0 + 0.01 * basic_motivation * (1.0 + 0.01 * buffs.ganjing as f32))
                    * (1.0 + 0.01 * buffs.xunlian as f32)
                    * (1.0 + 0.05 * person_count as f32)
                    * (1.0 + 0.01 * status_bonus[i] as f32))
                    .floor() as i32;
            }
        }
        // 智力回体
        if train == 4 {
            ret.vital += buffs.wiz_vital_bonus;
        }
        // 体力消耗降低
        if ret.vital < 0 {
            ret.vital = (ret.vital as f32 * (1.0 - 0.01 * buffs.vital_cost_drop)) as i32;
        }
        Ok(ret)
    }
}

pub trait Trainer<G: Game> {
    /// 根据当前局面选择动作
    fn select_action(&self, game: &G, actions: &[<G as Game>::Action], rng: &mut StdRng) -> Result<usize>;
}
