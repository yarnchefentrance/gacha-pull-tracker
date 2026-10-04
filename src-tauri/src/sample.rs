//! Deterministic sample histories so a fresh install (and the UI in dev) shows realistic content.
//! Pulls are simulated with the real pity rules, so the numbers on screen are statistically plausible.

use crate::models::{standard_pool, Account, BannerKind, Game, Pull};
use crate::odds::{rate_at, Rng};
use crate::pity::PityRules;

/// 2026-11-07 21:40:00 UTC, the newest sample pull.
const SAMPLE_END: i64 = 1_794_087_600;

struct Pools {
    featured_chars: &'static [&'static str],
    featured_weapons: &'static [&'static str],
    four_chars: &'static [&'static str],
    four_weapons: &'static [&'static str],
    three: &'static [&'static str],
    words: (&'static str, &'static str),
}

fn pools(game: Game) -> Pools {
    match game {
        Game::Genshin => Pools {
            featured_chars: &["Neuvillette", "Furina", "Arlecchino", "Mavuika", "Citlali", "Varesa", "Escoffier", "Ineffa", "Lauma", "Flins"],
            featured_weapons: &["Splendor of Tranquil Waters", "Crimson Moon's Semblance", "A Thousand Blazing Suns"],
            four_chars: &["Bennett", "Xiangling", "Xingqiu", "Fischl", "Sucrose", "Chevreuse", "Kachina", "Ororon"],
            four_weapons: &["Favonius Sword", "Sacrificial Bow", "The Widsith", "Rust", "Favonius Lance"],
            three: &["Cool Steel", "Harbinger of Dawn", "Slingshot", "Debate Club", "Black Tassel", "Raven Bow", "Magic Guide"],
            words: ("Character", "Weapon"),
        },
        Game::StarRail => Pools {
            featured_chars: &["Acheron", "Firefly", "Feixiao", "The Herta", "Aglaea", "Castorice", "Phainon"],
            featured_weapons: &["Along the Passing Shore", "Whereabouts Should Dreams Rest", "I Venture Forth to Hunt"],
            four_chars: &["Pela", "Tingyun", "Asta", "Sushang", "Hanya", "Gallagher", "Lynx"],
            four_weapons: &["Dance! Dance! Dance!", "Memories of the Past", "Planetary Rendezvous"],
            three: &["Arrows", "Cornucopia", "Collapsing Sky", "Amber", "Void", "Chorus", "Mutual Demise"],
            words: ("Character", "Light Cone"),
        },
        Game::Zenless => Pools {
            featured_chars: &["Ellen", "Zhu Yuan", "Jane", "Miyabi", "Evelyn", "Astra Yao", "Yixuan"],
            featured_weapons: &["Deep Sea Visitor", "Hailstorm Shrine", "Flamemaker Shaker"],
            four_chars: &["Nicole", "Anby", "Soukaku", "Lucy", "Ben", "Pulchra"],
            four_weapons: &["Steam Oven", "Bashful Demon", "Starlight Engine", "Slice of Time"],
            three: &["[Lunar] Pleniluna", "[Reverb] Mark I", "[Vortex] Arrow", "[Cannon] Rotor"],
            words: ("Agents", "W-Engines"),
        },
        Game::Wuthering => Pools {
            featured_chars: &["Jiyan", "Yinlin", "Camellya", "Carlotta", "Zani", "Cartethyia", "Phrolova"],
            featured_weapons: &["Verdant Summit", "Stringmaster", "Red Spring"],
            four_chars: &["Sanhua", "Baizhi", "Chixia", "Danjin", "Mortefi", "Yuanwu", "Taoqi"],
            four_weapons: &["Lumingloss", "Commando of Conviction", "Hollow Mirage"],
            three: &["Originite: Type I", "Guardian Broadblade", "Guardian Sword", "Guardian Pistols", "Guardian Rectifier"],
            words: ("Resonator", "Weapon"),
        },
    }
}

/// (five-star count, final pity, seed) per banner of each game.
fn plan(game: Game, banner: BannerKind) -> (u32, u32, u64) {
    use BannerKind::*;
    match (game, banner) {
        (Game::Genshin, Character) => (10, 67, 11),
        (Game::Genshin, Weapon) => (2, 31, 12),
        (Game::Genshin, Standard) => (3, 48, 13),
        (Game::Genshin, _) => (1, 12, 14),
        (Game::StarRail, Character) => (7, 54, 21),
        (Game::StarRail, Weapon) => (3, 22, 22),
        (Game::StarRail, _) => (2, 71, 23),
        (Game::Zenless, Character) => (6, 38, 31),
        (Game::Zenless, Weapon) => (2, 60, 32),
        (Game::Zenless, Standard) => (2, 15, 33),
        (Game::Zenless, _) => (1, 9, 34),
        (Game::Wuthering, Character) => (5, 44, 41),
        (Game::Wuthering, Weapon) => (2, 18, 42),
        (Game::Wuthering, _) => (1, 36, 43),
    }
}

fn pick<'a>(rng: &mut Rng, list: &'a [&'a str]) -> &'a str {
    list[rng.below(list.len())]
}

/// Simulates one banner and returns pulls oldest-first without timestamps.
fn simulate(game: Game, banner: BannerKind) -> Vec<(String, String, u8, Option<bool>)> {
    let (fives, final_pity, seed) = plan(game, banner);
    let rules = PityRules::for_banner(game, banner);
    let pool = pools(game);
    let mut rng = Rng::new(seed);
    let weapon_banner = banner == BannerKind::Weapon;
    let (char_word, weapon_word) = pool.words;
    let mut out = Vec::new();
    let (mut pity, mut pity4, mut got, mut guaranteed, mut featured_idx) = (0u32, 0u32, 0u32, false, 0usize);
    let featured_list = if weapon_banner { pool.featured_weapons } else { pool.featured_chars };
    loop {
        let finishing = got == fives;
        if finishing && pity == final_pity {
            break;
        }
        pity += 1;
        pity4 += 1;
        let five = !finishing && rng.next_f64() < rate_at(&rules, pity);
        if five {
            let std_pool = standard_pool(game);
            let win = guaranteed || !rules.has_fifty_fifty() || rng.next_f64() < rules.featured_rate;
            let (name, featured) = if banner == BannerKind::Bangboo {
                (pick(&mut rng, &["Butler", "Sharkboo", "Plugboo", "Amillion"]).to_string(), None)
            } else if banner == BannerKind::Standard {
                (pick(&mut rng, std_pool).to_string(), None)
            } else if win {
                featured_idx = (featured_idx + 1 + rng.below(2)) % featured_list.len();
                (featured_list[featured_idx].to_string(), Some(true))
            } else {
                (pick(&mut rng, std_pool).to_string(), Some(false))
            };
            guaranteed = !win && rules.carries_guarantee;
            let kind = match banner {
                BannerKind::Weapon => weapon_word,
                BannerKind::Bangboo => "Bangboo",
                _ => char_word,
            };
            out.push((name, kind.to_string(), 5, featured));
            got += 1;
            pity = 0;
            pity4 = 0;
        } else if pity4 >= 10 || rng.next_f64() < 0.051 {
            let as_char = !weapon_banner && rng.below(3) > 0;
            let (name, kind) = if as_char { (pick(&mut rng, pool.four_chars), char_word) } else { (pick(&mut rng, pool.four_weapons), weapon_word) };
            out.push((name.to_string(), kind.to_string(), 4, None));
            pity4 = 0;
        } else {
            out.push((pick(&mut rng, pool.three).to_string(), weapon_word.to_string(), 3, None));
        }
        if finishing && pity == final_pity {
            break;
        }
    }
    out
}

/// Converts a unix timestamp to `YYYY-MM-DD HH:MM:SS` (UTC, proleptic Gregorian).
pub fn format_time(ts: i64) -> String {
    let days = ts.div_euclid(86_400);
    let secs = ts.rem_euclid(86_400);
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}", secs / 3600, secs % 3600 / 60, secs % 60)
}

pub fn history(game: Game) -> Vec<Pull> {
    let mut all = Vec::new();
    for (bi, banner) in game.banners().iter().enumerate() {
        let sim = simulate(game, *banner);
        let mut rng = Rng::new(900 + bi as u64 + game as u64 * 10);
        // Assign timestamps backwards: every 10 pulls share one timestamp, sessions are 4-60 h apart.
        let mut t = SAMPLE_END - bi as i64 * 5_400;
        let mut stamps = vec![0i64; sim.len()];
        for (i, slot) in stamps.iter_mut().rev().enumerate() {
            if i > 0 && i % 10 == 0 {
                t -= 14_400 + rng.below(200_000) as i64;
            }
            *slot = t;
        }
        for (i, ((name, kind, rarity, featured), ts)) in sim.into_iter().zip(stamps).enumerate() {
            all.push(Pull {
                id: format!("{}{:03}{:03}", ts, bi, i % 1000),
                time: format_time(ts),
                name,
                item_type: kind,
                rarity,
                banner: *banner,
                featured,
            });
        }
    }
    all.sort_by(|a, b| a.time.cmp(&b.time).then_with(|| a.id.cmp(&b.id)));
    all
}

pub fn account(game: Game) -> Account {
    let (uid, region, timezone) = match game {
        Game::Genshin => ("712408113", "os_euro", 1),
        Game::StarRail => ("704918822", "prod_official_eur", 1),
        Game::Zenless => ("1500390471", "prod_gf_eu", 1),
        Game::Wuthering => ("500127764", "Europe", 1),
    };
    Account { game, uid: uid.into(), region: region.into(), timezone, lang: "en-us".into() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pity::analyze;

    #[test]
    fn formats_unix_time() {
        assert_eq!(format_time(0), "1970-01-01 00:00:00");
        assert_eq!(format_time(SAMPLE_END), "2026-11-07 21:40:00");
        assert_eq!(format_time(951_782_400), "2000-02-29 00:00:00");
    }

    #[test]
    fn sample_matches_plan() {
        for g in Game::ALL {
            let h = history(g);
            for b in g.banners() {
                let (fives, pity, _) = plan(g, *b);
                let st = analyze(g, *b, &h);
                assert_eq!(st.five_stars, fives, "{g:?} {b:?}");
                assert_eq!(st.pity, pity, "{g:?} {b:?}");
                assert!(st.hits.iter().all(|x| x.pity <= st.rules.hard_pity));
            }
        }
    }
}
