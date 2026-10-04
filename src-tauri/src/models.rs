//! Core domain model shared by the pity engine, odds engine, importer and UIGF exporter.

use serde::{Deserialize, Serialize};

/// Supported games. Serialized as short lowercase ids used by the UI and the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Game {
    #[serde(rename = "genshin")]
    Genshin,
    #[serde(rename = "hsr")]
    StarRail,
    #[serde(rename = "zzz")]
    Zenless,
    #[serde(rename = "wuwa")]
    Wuthering,
}

impl Game {
    pub const ALL: [Game; 4] = [Game::Genshin, Game::StarRail, Game::Zenless, Game::Wuthering];

    pub fn parse(id: &str) -> Option<Game> {
        match id.trim().to_ascii_lowercase().as_str() {
            "genshin" | "gi" | "hk4e" => Some(Game::Genshin),
            "hsr" | "starrail" | "hkrpg" => Some(Game::StarRail),
            "zzz" | "nap" | "zenless" => Some(Game::Zenless),
            "wuwa" | "ww" | "wuthering" => Some(Game::Wuthering),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Game::Genshin => "genshin",
            Game::StarRail => "hsr",
            Game::Zenless => "zzz",
            Game::Wuthering => "wuwa",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Game::Genshin => "Genshin Impact",
            Game::StarRail => "Honkai: Star Rail",
            Game::Zenless => "Zenless Zone Zero",
            Game::Wuthering => "Wuthering Waves",
        }
    }

    /// What the game calls a single pull.
    pub fn pull_word(self) -> &'static str {
        match self {
            Game::Genshin => "Wish",
            Game::StarRail => "Warp",
            Game::Zenless => "Signal Search",
            Game::Wuthering => "Convene",
        }
    }

    /// Premium currency used by the savings planner.
    pub fn currency(self) -> &'static str {
        match self {
            Game::Genshin => "Primogems",
            Game::StarRail => "Stellar Jade",
            Game::Zenless => "Polychrome",
            Game::Wuthering => "Astrite",
        }
    }

    /// UIGF v4 section key, if the game is covered by the standard.
    pub fn uigf_key(self) -> Option<&'static str> {
        match self {
            Game::Genshin => Some("hk4e"),
            Game::StarRail => Some("hkrpg"),
            Game::Zenless => Some("nap"),
            Game::Wuthering => None,
        }
    }

    pub fn banners(self) -> &'static [BannerKind] {
        use BannerKind::*;
        match self {
            Game::Genshin => &[Character, Weapon, Standard, Chronicled],
            Game::StarRail => &[Character, Weapon, Standard],
            Game::Zenless => &[Character, Weapon, Standard, Bangboo],
            Game::Wuthering => &[Character, Weapon, Standard],
        }
    }
}

/// Banner families. Concrete names differ per game, see [`BannerKind::label`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BannerKind {
    Character,
    Weapon,
    Standard,
    Chronicled,
    Bangboo,
}

impl BannerKind {
    pub fn parse(id: &str) -> Option<BannerKind> {
        match id {
            "character" => Some(BannerKind::Character),
            "weapon" => Some(BannerKind::Weapon),
            "standard" => Some(BannerKind::Standard),
            "chronicled" => Some(BannerKind::Chronicled),
            "bangboo" => Some(BannerKind::Bangboo),
            _ => None,
        }
    }

    pub fn label(self, game: Game) -> &'static str {
        use BannerKind::*;
        match (game, self) {
            (Game::Genshin, Character) => "Character Event Wish",
            (Game::Genshin, Weapon) => "Weapon Event Wish",
            (Game::Genshin, Standard) => "Wanderlust Invocation",
            (Game::Genshin, Chronicled) => "Chronicled Wish",
            (Game::StarRail, Character) => "Character Event Warp",
            (Game::StarRail, Weapon) => "Light Cone Event Warp",
            (Game::StarRail, _) => "Stellar Warp",
            (Game::Zenless, Character) => "Exclusive Channel",
            (Game::Zenless, Weapon) => "W-Engine Channel",
            (Game::Zenless, Bangboo) => "Bangboo Channel",
            (Game::Zenless, _) => "Stable Channel",
            (Game::Wuthering, Character) => "Featured Resonator Convene",
            (Game::Wuthering, Weapon) => "Featured Weapon Convene",
            (Game::Wuthering, _) => "Standard Resonator Convene",
            (_, _) => "Event Banner",
        }
    }

    /// Raw `gacha_type` values used by the official history endpoints.
    pub fn gacha_types(self, game: Game) -> &'static [&'static str] {
        use BannerKind::*;
        match (game, self) {
            (Game::Genshin, Character) => &["301", "400"],
            (Game::Genshin, Weapon) => &["302"],
            (Game::Genshin, Standard) => &["200"],
            (Game::Genshin, Chronicled) => &["500"],
            (Game::StarRail, Character) => &["11"],
            (Game::StarRail, Weapon) => &["12"],
            (Game::StarRail, _) => &["1"],
            (Game::Zenless, Character) => &["2"],
            (Game::Zenless, Weapon) => &["3"],
            (Game::Zenless, Bangboo) => &["5"],
            (Game::Zenless, _) => &["1"],
            (Game::Wuthering, Character) => &["1"],
            (Game::Wuthering, Weapon) => &["2"],
            (Game::Wuthering, _) => &["3"],
            (_, _) => &[],
        }
    }

    pub fn from_gacha_type(game: Game, raw: &str) -> Option<BannerKind> {
        game.banners().iter().copied().find(|b| b.gacha_types(game).contains(&raw))
    }
}

/// One pull as stored locally. Pulls are kept oldest-first per banner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pull {
    pub id: String,
    /// Server-local time, `YYYY-MM-DD HH:MM:SS`.
    pub time: String,
    pub name: String,
    pub item_type: String,
    pub rarity: u8,
    pub banner: BannerKind,
    /// `Some(true)` = rate-up item, `Some(false)` = off-banner. `None` = derive from the standard pool list.
    #[serde(default)]
    pub featured: Option<bool>,
}

/// Items that can be lost to on a 50/50. Used when the history source does not flag rate-up items.
pub fn standard_pool(game: Game) -> &'static [&'static str] {
    match game {
        Game::Genshin => &[
            "Diluc", "Jean", "Keqing", "Mona", "Qiqi", "Tighnari", "Dehya", "Yumemizuki Mizuki",
            "Wolf's Gravestone", "Skyward Harp", "Skyward Pride", "Amos' Bow", "Aquila Favonia",
            "Primordial Jade Winged-Spear", "Lost Prayer to the Sacred Winds", "Skyward Atlas",
        ],
        Game::StarRail => &[
            "Himeko", "Welt", "Bronya", "Gepard", "Clara", "Yanqing", "Bailu",
            "Night on the Milky Way", "Something Irreplaceable", "But the Battle Isn't Over",
            "In the Name of the World", "Moment of Victory", "Sleep Like the Dead", "Time Waits for No One",
        ],
        Game::Zenless => &[
            "Grace", "Rina", "Koleda", "Nekomata", "Soldier 11", "Lycaon",
            "Steel Cushion", "The Restrained", "Fusion Compiler", "Weeping Cradle", "Hellfire Gears", "The Brimstone",
        ],
        Game::Wuthering => &["Verina", "Encore", "Calcharo", "Lingyang", "Jianxin"],
    }
}

pub fn is_standard_item(game: Game, name: &str) -> bool {
    standard_pool(game).iter().any(|n| n.eq_ignore_ascii_case(name))
}

/// Account metadata shown in the header and written into UIGF exports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub game: Game,
    pub uid: String,
    pub region: String,
    pub timezone: i32,
    pub lang: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_ids_round_trip() {
        for g in Game::ALL {
            assert_eq!(Game::parse(g.id()), Some(g));
        }
        assert_eq!(Game::parse("HK4E"), Some(Game::Genshin));
        assert_eq!(Game::parse("nope"), None);
    }

    #[test]
    fn gacha_type_mapping() {
        assert_eq!(BannerKind::from_gacha_type(Game::Genshin, "400"), Some(BannerKind::Character));
        assert_eq!(BannerKind::from_gacha_type(Game::StarRail, "12"), Some(BannerKind::Weapon));
        assert_eq!(BannerKind::from_gacha_type(Game::Zenless, "5"), Some(BannerKind::Bangboo));
        assert_eq!(BannerKind::from_gacha_type(Game::Genshin, "999"), None);
    }

    #[test]
    fn standard_pool_is_case_insensitive() {
        assert!(is_standard_item(Game::Genshin, "diluc"));
        assert!(!is_standard_item(Game::Genshin, "Furina"));
    }
}
