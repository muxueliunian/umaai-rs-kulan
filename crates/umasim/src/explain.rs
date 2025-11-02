use colored::Colorize;

use crate::utils::{Array5, Array6};

pub struct Explain;

/// 解释一些基础类型的值
impl Explain {
    // 后面可能用文件表示
    const MOTIVATION: [&'static str; 5] = ["绝不调", "不调", "普通", "好调", "绝好调"];
    const FIVE_STATUS: [&'static str; 5] = ["速", "耐", "力", "根", "智"];
    pub fn motivation(m: i32) -> &'static str {
        if m == 0 {
            "干劲错误"
        } else {
            Self::MOTIVATION.get((m - 1) as usize).cloned().unwrap_or("干劲错误")
        }
    }

    pub fn five_status(stats: &Array5) -> String {
        let mut s = String::new();
        for (i, stat) in stats.iter().enumerate() {
            s += &format!("{}{} ", Self::FIVE_STATUS[i], stat);
        }
        s
    }

    pub fn five_status_cutted(stats: &Array5) -> String {
        let mut s = String::new();
        for (i, stat) in stats.iter().enumerate() {
            if *stat <= 1200 {
                s += &format!("{}{} ", Self::FIVE_STATUS[i], stat);
            } else {
                let cutted = 1200 + (stat - 1200) / 2;
                s += &format!("{}", format!("{}{} ", Self::FIVE_STATUS[i], cutted).cyan());
            }
        }
        s
    }

    pub fn status_with_pt(stats: &Array6) -> String {
        format!(
            "{}{}pt",
            Self::five_status(&stats[..5].try_into().expect("status-with-pt")),
            stats[5]
        )
    }

    pub fn train_level_count(train: &Array5) -> String {
        let levels: Vec<_> = train.iter().map(|x| (x / 4 + 1).min(5)).collect();
        format!("{levels:?}")
    }
}
