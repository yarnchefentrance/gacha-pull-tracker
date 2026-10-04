//! Pity engine: per-game banner rules and history analysis (pity counters, 50/50 outcomes, guarantees).

use crate::models::{is_standard_item, BannerKind, Game, Pull};
use serde::Serialize;

/// Published (and community-measured) rate rules for one banner.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct PityRules {
    /// Pull number at which a 5-star is guaranteed (90 or 80).
    pub hard_pity: u32,
    /// First pull number where the rate starts climbing.
    pub soft_start: u32,
    /// Base 5-star rate per pull.
    pub base_rate: f64,
    /// Rate added per pull from `soft_start` on.
    pub soft_step: f64,
    /// 4-star guarantee interval.
    pub four_star_pity: u32,
    /// Chance that a 5-star is the featured item when no guarantee is active (0.5, 0.75 or 1.0).
    pub featured_rate: f64,
    /// Whether losing the featured roll guarantees the next 5-star.
    pub carries_guarantee: bool,
    /// Genshin 5.0+ Capturing Radiance mechanic applies to this banner.
    pub capturing_radiance: bool,
}

impl PityRules {
    pub fn for_banner(game: Game, banner: BannerKind) -> PityRules {
        use BannerKind::*;
        let base = PityRules {
            hard_pity: 90,
            soft_start: 74,
            base_rate: 0.006,
            soft_step: 0.06,
            four_star_pity: 10,
            featured_rate: 0.5,
            carries_guarantee: true,
            capturing_radiance: false,
        };
        let weapon = PityRules { hard_pity: 80, featured_rate: 0.75, ..base };
        let standard = PityRules { featured_rate: 1.0, carries_guarantee: false, ..base };
        match (game, banner) {
            (Game::Genshin, Character) => PityRules { capturing_radiance: true, ..base },
            (Game::Genshin, Weapon) => PityRules { soft_start: 63, base_rate: 0.007, soft_step: 0.07, ..weapon },
            (Game::Genshin, Chronicled) => base,
            (Game::StarRail, Character) => base,
            (Game::StarRail, Weapon) => PityRules { soft_start: 66, base_rate: 0.008, soft_step: 0.07, ..weapon },
            (Game::Zenless, Character) => base,
            (Game::Zenless, Weapon) => PityRules { soft_start: 65, base_rate: 0.01, ..weapon },
            (Game::Zenless, Bangboo) => PityRules { hard_pity: 80, soft_start: 65, base_rate: 0.01, ..standard },
            (Game::Wuthering, Character) => PityRules { hard_pity: 80, soft_start: 66, base_rate: 0.008, soft_step: 0.07, ..base },
            (Game::Wuthering, Weapon) => PityRules {
                hard_pity: 80,
                soft_start: 66,
                base_rate: 0.008,
                soft_step: 0.07,
                featured_rate: 1.0,
                carries_guarantee: false,
                ..base
            },
            (Game::Wuthering, _) => PityRules { hard_pity: 80, soft_start: 66, base_rate: 0.008, soft_step: 0.07, ..standard },
            (_, _) => standard,
        }
    }

    /// Whether this banner has a featured/off-banner split at all.
    pub fn has_fifty_fifty(&self) -> bool {
        self.featured_rate < 1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    /// Won the featured roll.
    Won,
    /// Lost the featured roll (got an off-banner item).
    Lost,
    /// Featured item via guarantee after a loss.
    Guaranteed,
    /// Banner without a featured split.
    Standard,
}

#[derive(Debug, Clone, Serialize)]
pub struct FiveStarHit {
    pub name: String,
    pub time: String,
    pub pity: u32,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, Serialize)]
pub struct BannerStats {
    pub banner: BannerKind,
    pub label: String,
    pub rules: PityRules,
    pub total: u32,
    pub five_stars: u32,
    pub four_stars: u32,
    /// Pulls since the last 5-star.
    pub pity: u32,
    /// Pulls since the last 4-star or better.
    pub pity4: u32,
    /// Next 5-star is guaranteed featured.
    pub guaranteed: bool,
    pub in_soft_pity: bool,
    pub avg_pity: f64,
    pub wins: u32,
    pub losses: u32,
    /// Consecutive lost 50/50s (Capturing Radiance counter, Genshin only).
    pub lost_streak: u32,
    /// Share of 50/50s won, guaranteed pulls excluded.
    pub win_rate: Option<f64>,
    pub hits: Vec<FiveStarHit>,
    pub notes: Vec<String>,
}

fn is_featured(game: Game, p: &Pull) -> bool {
    p.featured.unwrap_or_else(|| !is_standard_item(game, &p.name))
}

/// Walks a banner history (oldest first) and derives every counter the UI shows.
pub fn analyze(game: Game, banner: BannerKind, pulls: &[Pull]) -> BannerStats {
    let rules = PityRules::for_banner(game, banner);
    let mut st = BannerStats {
        banner,
        label: banner.label(game).to_string(),
        rules,
        total: 0,
        five_stars: 0,
        four_stars: 0,
        pity: 0,
        pity4: 0,
        guaranteed: false,
        in_soft_pity: false,
        avg_pity: 0.0,
        wins: 0,
        losses: 0,
        lost_streak: 0,
        win_rate: None,
        hits: Vec::new(),
        notes: Vec::new(),
    };
    let mut pity_sum = 0u64;
    for p in pulls.iter().filter(|p| p.banner == banner) {
        st.total += 1;
        st.pity += 1;
        st.pity4 += 1;
        match p.rarity {
            5 => {
                let outcome = if !rules.has_fifty_fifty() {
                    Outcome::Standard
                } else if st.guaranteed {
                    Outcome::Guaranteed
                } else if is_featured(game, p) {
                    Outcome::Won
                } else {
                    Outcome::Lost
                };
                match outcome {
                    Outcome::Won => {
                        st.wins += 1;
                        st.lost_streak = 0;
                    }
                    Outcome::Lost => {
                        st.losses += 1;
                        st.lost_streak += 1;
                    }
                    _ => {}
                }
                st.guaranteed = outcome == Outcome::Lost && rules.carries_guarantee;
                st.five_stars += 1;
                pity_sum += st.pity as u64;
                st.hits.push(FiveStarHit { name: p.name.clone(), time: p.time.clone(), pity: st.pity, outcome });
                st.pity = 0;
                st.pity4 = 0;
            }
            4 => {
                st.four_stars += 1;
                st.pity4 = 0;
            }
            _ => {}
        }
    }
    if st.five_stars > 0 {
        st.avg_pity = pity_sum as f64 / st.five_stars as f64;
    }
    st.in_soft_pity = st.pity + 1 >= rules.soft_start;
    let rolls = st.wins + st.losses;
    st.win_rate = (rolls > 0).then(|| st.wins as f64 / rolls as f64);
    st.notes = build_notes(&st);
    st
}

fn build_notes(st: &BannerStats) -> Vec<String> {
    let r = &st.rules;
    let mut notes = Vec::new();
    let left = r.hard_pity.saturating_sub(st.pity);
    if st.in_soft_pity {
        notes.push(format!("Soft pity active: rate is climbing, hard pity in {left} pulls."));
    } else {
        let to_soft = r.soft_start.saturating_sub(st.pity);
        notes.push(format!("{to_soft} pulls to soft pity (pull {}), {left} to hard pity.", r.soft_start));
    }
    if st.guaranteed {
        notes.push("Next 5-star is guaranteed to be the featured item.".to_string());
    } else if r.has_fifty_fifty() {
        notes.push(format!("Next 5-star is a {:.0}/{:.0}.", r.featured_rate * 100.0, (1.0 - r.featured_rate) * 100.0));
    }
    if r.capturing_radiance {
        // The exact formula is not published; the community model is a counter that forces
        // a win after enough consecutive losses. Shown as information only.
        notes.push(format!(
            "Capturing Radiance: {} consecutive lost 50/50s on record. Long losing streaks raise the chance to win the next one.",
            st.lost_streak
        ));
    }
    notes
}

/// Analyzes every banner of a game in display order.
pub fn analyze_game(game: Game, pulls: &[Pull]) -> Vec<BannerStats> {
    game.banners().iter().map(|b| analyze(game, *b, pulls)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pull(name: &str, rarity: u8, banner: BannerKind) -> Pull {
        Pull {
            id: String::new(),
            time: "2026-10-01 12:00:00".into(),
            name: name.into(),
            item_type: "Character".into(),
            rarity,
            banner,
            featured: None,
        }
    }

    fn run(n_three: usize, last: (&str, u8)) -> Vec<Pull> {
        let mut v: Vec<Pull> = (0..n_three).map(|_| pull("Cool Steel", 3, BannerKind::Character)).collect();
        v.push(pull(last.0, last.1, BannerKind::Character));
        v
    }

    #[test]
    fn rules_hard_pity() {
        assert_eq!(PityRules::for_banner(Game::Genshin, BannerKind::Character).hard_pity, 90);
        assert_eq!(PityRules::for_banner(Game::Genshin, BannerKind::Weapon).hard_pity, 80);
        assert_eq!(PityRules::for_banner(Game::Wuthering, BannerKind::Character).hard_pity, 80);
        assert!(!PityRules::for_banner(Game::Wuthering, BannerKind::Weapon).has_fifty_fifty());
        assert!(PityRules::for_banner(Game::Genshin, BannerKind::Character).capturing_radiance);
    }

    #[test]
    fn counts_pity_since_last_five_star() {
        let mut v = run(76, ("Furina", 5));
        v.extend((0..12).map(|_| pull("Cool Steel", 3, BannerKind::Character)));
        let st = analyze(Game::Genshin, BannerKind::Character, &v);
        assert_eq!(st.total, 89);
        assert_eq!(st.five_stars, 1);
        assert_eq!(st.hits[0].pity, 77);
        assert_eq!(st.pity, 12);
        assert_eq!(st.pity4, 12);
        assert_eq!(st.hits[0].outcome, Outcome::Won);
        assert!(!st.guaranteed);
    }

    #[test]
    fn lost_fifty_fifty_sets_guarantee() {
        let mut v = run(70, ("Diluc", 5));
        let st = analyze(Game::Genshin, BannerKind::Character, &v);
        assert_eq!(st.hits[0].outcome, Outcome::Lost);
        assert!(st.guaranteed);
        assert_eq!(st.lost_streak, 1);

        v.extend(run(40, ("Mavuika", 5)));
        let st = analyze(Game::Genshin, BannerKind::Character, &v);
        assert_eq!(st.hits[1].outcome, Outcome::Guaranteed);
        assert!(!st.guaranteed);
        assert_eq!((st.wins, st.losses), (0, 1));
        assert_eq!(st.lost_streak, 1, "guaranteed pulls do not reset the radiance counter");
        assert!((st.avg_pity - 56.0).abs() < 1e-9);
    }

    #[test]
    fn explicit_featured_flag_wins_over_name_list() {
        let mut v = run(10, ("Diluc", 5));
        v.last_mut().unwrap().featured = Some(true);
        let st = analyze(Game::Genshin, BannerKind::Character, &v);
        assert_eq!(st.hits[0].outcome, Outcome::Won);
    }

    #[test]
    fn four_star_resets_only_four_star_pity() {
        let v = run(5, ("Bennett", 4));
        let st = analyze(Game::Genshin, BannerKind::Character, &v);
        assert_eq!(st.pity, 6);
        assert_eq!(st.pity4, 0);
        assert_eq!(st.four_stars, 1);
    }

    #[test]
    fn standard_banner_has_no_fifty_fifty() {
        let v: Vec<Pull> = run(50, ("Diluc", 5)).into_iter().map(|mut p| { p.banner = BannerKind::Standard; p }).collect();
        let st = analyze(Game::Genshin, BannerKind::Standard, &v);
        assert_eq!(st.hits[0].outcome, Outcome::Standard);
        assert!(!st.guaranteed);
        assert_eq!(st.win_rate, None);
    }

    #[test]
    fn soft_pity_flag() {
        let v: Vec<Pull> = (0..73).map(|_| pull("Cool Steel", 3, BannerKind::Character)).collect();
        let st = analyze(Game::Genshin, BannerKind::Character, &v);
        assert!(st.in_soft_pity, "pull 74 is the next one");
        let st = analyze(Game::Genshin, BannerKind::Character, &v[..60]);
        assert!(!st.in_soft_pity);
    }

    #[test]
    fn other_banners_are_ignored() {
        let mut v = run(3, ("Furina", 5));
        v.push(pull("Wolf's Gravestone", 5, BannerKind::Weapon));
        let st = analyze(Game::Genshin, BannerKind::Character, &v);
        assert_eq!(st.total, 4);
        assert_eq!(analyze_game(Game::Genshin, &v).len(), 4);
    }
}
