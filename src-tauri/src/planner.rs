//! Savings planner: projects premium currency income until a target date and converts it into pulls
//! and into the probability of getting the featured 5-star.

use crate::models::{BannerKind, Game};
use crate::odds;
use crate::pity::PityRules;
use serde::{Deserialize, Serialize};

/// Currency cost of one pull. 160 in all four games.
pub const PULL_COST: u32 = 160;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanInput {
    pub game: Game,
    pub banner: BannerKind,
    /// Premium currency on hand (primogems / jades / polychromes / astrite).
    pub currency: u32,
    /// Limited pull items on hand (fates, passes, tapes, tides).
    pub pulls_owned: u32,
    /// Days until the target banner.
    pub days: u32,
    pub daily_missions: bool,
    pub monthly_pass: bool,
    pub battle_pass: bool,
    /// Average endgame clear rate, 0.0 - 1.0.
    pub endgame_clear: f64,
    /// Estimated currency from events and patch rewards per 42-day version.
    pub events_per_patch: u32,
    pub pity: u32,
    pub guaranteed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct IncomeLine {
    pub label: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlanResult {
    pub currency_total: u32,
    pub pulls: u32,
    pub lines: Vec<IncomeLine>,
    pub p_any: f64,
    pub p_featured: f64,
    /// Extra pulls needed for a 90% chance of the featured item (0 if already there).
    pub shortfall_90: u32,
    pub per_day: f64,
}

/// Rough per-game income table (daily, monthly pass per day, endgame per cycle, cycle days, battle pass per patch).
struct Income {
    daily: u32,
    pass_daily: u32,
    endgame: u32,
    endgame_cycle: u32,
    battle_pass_pulls: u32,
}

fn income(game: Game) -> Income {
    match game {
        Game::Genshin => Income { daily: 60, pass_daily: 90, endgame: 1600, endgame_cycle: 30, battle_pass_pulls: 4 },
        Game::StarRail => Income { daily: 60, pass_daily: 90, endgame: 2400, endgame_cycle: 42, battle_pass_pulls: 4 },
        Game::Zenless => Income { daily: 60, pass_daily: 90, endgame: 1560, endgame_cycle: 28, battle_pass_pulls: 4 },
        Game::Wuthering => Income { daily: 60, pass_daily: 90, endgame: 1600, endgame_cycle: 28, battle_pass_pulls: 4 },
    }
}

pub fn plan(input: &PlanInput) -> PlanResult {
    let inc = income(input.game);
    let d = input.days;
    let mut lines = vec![IncomeLine { label: format!("{} on hand", input.game.currency()), amount: input.currency }];
    if input.pulls_owned > 0 {
        lines.push(IncomeLine { label: "Pull items on hand".into(), amount: input.pulls_owned * PULL_COST });
    }
    if input.daily_missions {
        lines.push(IncomeLine { label: format!("Daily missions ({d} days)"), amount: inc.daily * d });
    }
    if input.monthly_pass {
        lines.push(IncomeLine { label: "Monthly pass".into(), amount: inc.pass_daily * d });
    }
    let cycles = d / inc.endgame_cycle;
    if cycles > 0 && input.endgame_clear > 0.0 {
        let amount = (inc.endgame as f64 * input.endgame_clear.clamp(0.0, 1.0) * cycles as f64).round() as u32;
        lines.push(IncomeLine { label: format!("Endgame modes ({cycles} resets)"), amount });
    }
    let patches = d as f64 / 42.0;
    if input.events_per_patch > 0 {
        lines.push(IncomeLine { label: "Events and patch rewards".into(), amount: (input.events_per_patch as f64 * patches).round() as u32 });
    }
    if input.battle_pass {
        let n = (inc.battle_pass_pulls as f64 * patches).floor() as u32;
        lines.push(IncomeLine { label: "Battle pass pulls".into(), amount: n * PULL_COST });
    }
    let total: u32 = lines.iter().map(|l| l.amount).sum();
    let pulls = total / PULL_COST;
    let rules = PityRules::for_banner(input.game, input.banner);
    let p_featured = odds::p_featured_within(&rules, input.pity, input.guaranteed, pulls);
    let need = odds::pulls_for_confidence(&rules, input.pity, input.guaranteed, 0.9);
    PlanResult {
        currency_total: total,
        pulls,
        lines,
        p_any: odds::p_within(&rules, input.pity, pulls),
        p_featured,
        shortfall_90: need.saturating_sub(pulls),
        per_day: if d == 0 { 0.0 } else { (total - input.currency - input.pulls_owned * PULL_COST) as f64 / d as f64 },
    }
}

impl PlanInput {
    /// Planner defaults shown before the user edits anything.
    pub fn preset(game: Game) -> PlanInput {
        PlanInput {
            game,
            banner: BannerKind::Character,
            currency: 12_480,
            pulls_owned: 14,
            days: 23,
            daily_missions: true,
            monthly_pass: true,
            battle_pass: false,
            endgame_clear: 0.9,
            events_per_patch: 4_200,
            pity: 0,
            guaranteed: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_days_is_just_savings() {
        let mut i = PlanInput::preset(Game::Genshin);
        i.days = 0;
        let r = plan(&i);
        assert_eq!(r.currency_total, 12_480 + 14 * 160);
        assert_eq!(r.pulls, 92);
        assert_eq!(r.per_day, 0.0);
    }

    #[test]
    fn income_grows_with_days() {
        let mut i = PlanInput::preset(Game::StarRail);
        let short = plan(&i);
        i.days = 84;
        let long = plan(&i);
        assert!(long.pulls > short.pulls);
        assert!(long.p_featured >= short.p_featured);
        assert!(long.lines.iter().any(|l| l.label.starts_with("Endgame")));
    }

    #[test]
    fn shortfall_disappears_with_guarantee_and_enough_pulls() {
        let mut i = PlanInput::preset(Game::Zenless);
        i.guaranteed = true;
        i.pity = 60;
        let r = plan(&i);
        assert_eq!(r.shortfall_90, 0);
        assert!(r.p_featured > 0.99);
    }
}
