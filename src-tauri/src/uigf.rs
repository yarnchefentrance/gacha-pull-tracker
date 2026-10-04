//! UIGF v4.0 (Uniformed Interchangeable GachaLog Format) import / export.
//! Genshin = `hk4e`, Star Rail = `hkrpg`, ZZZ = `nap`. Wuthering Waves is not part of UIGF, so it is
//! written to a vendor extension section `x_wuwa` with the same record layout; other tools ignore it.

use crate::models::{Account, BannerKind, Game, Pull};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum UigfError {
    #[error("not a UIGF file: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("unsupported UIGF version {0}, expected v4.x")]
    Version(String),
    #[error("record {id} has an unknown gacha_type {gacha_type}")]
    GachaType { id: String, gacha_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub export_timestamp: serde_json::Value,
    pub export_app: String,
    pub export_app_version: String,
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Record {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uigf_gacha_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gacha_id: Option<String>,
    pub gacha_type: String,
    #[serde(default)]
    pub item_id: String,
    #[serde(default = "one")]
    pub count: String,
    pub time: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub item_type: String,
    pub rank_type: String,
    pub id: String,
}

fn one() -> String {
    "1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub uid: serde_json::Value,
    #[serde(default)]
    pub timezone: i32,
    #[serde(default)]
    pub lang: String,
    pub list: Vec<Record>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uigf {
    pub info: Info,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hk4e: Vec<Section>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hkrpg: Vec<Section>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nap: Vec<Section>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub x_wuwa: Vec<Section>,
}

impl Uigf {
    fn sections_mut(&mut self, game: Game) -> &mut Vec<Section> {
        match game {
            Game::Genshin => &mut self.hk4e,
            Game::StarRail => &mut self.hkrpg,
            Game::Zenless => &mut self.nap,
            Game::Wuthering => &mut self.x_wuwa,
        }
    }

    fn sections(&self, game: Game) -> &[Section] {
        match game {
            Game::Genshin => &self.hk4e,
            Game::StarRail => &self.hkrpg,
            Game::Zenless => &self.nap,
            Game::Wuthering => &self.x_wuwa,
        }
    }
}

fn to_record(game: Game, p: &Pull) -> Record {
    let gacha_type = p.banner.gacha_types(game).first().copied().unwrap_or("0").to_string();
    Record {
        uigf_gacha_type: (game == Game::Genshin).then(|| if gacha_type == "400" { "301".into() } else { gacha_type.clone() }),
        gacha_id: (game != Game::Genshin).then(String::new),
        gacha_type,
        item_id: String::new(),
        count: one(),
        time: p.time.clone(),
        name: p.name.clone(),
        item_type: p.item_type.clone(),
        rank_type: p.rarity.to_string(),
        id: p.id.clone(),
    }
}

/// Builds a UIGF document from the local store. `now` is a unix timestamp.
pub fn export(accounts: &[Account], histories: &BTreeMap<Game, Vec<Pull>>, now: u64) -> Uigf {
    let mut doc = Uigf {
        info: Info {
            export_timestamp: now.into(),
            export_app: "Wishboard".into(),
            export_app_version: env!("CARGO_PKG_VERSION").into(),
            version: "v4.0".into(),
        },
        hk4e: Vec::new(),
        hkrpg: Vec::new(),
        nap: Vec::new(),
        x_wuwa: Vec::new(),
    };
    for acc in accounts {
        let Some(pulls) = histories.get(&acc.game) else { continue };
        let mut list: Vec<Record> = pulls.iter().map(|p| to_record(acc.game, p)).collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        doc.sections_mut(acc.game).push(Section {
            uid: serde_json::Value::String(acc.uid.clone()),
            timezone: acc.timezone,
            lang: acc.lang.clone(),
            list,
        });
    }
    doc
}

pub fn to_json(doc: &Uigf) -> String {
    serde_json::to_string_pretty(doc).unwrap_or_default()
}

/// Imported history for one game account.
#[derive(Debug, Clone, Serialize)]
pub struct ImportedAccount {
    pub game: Game,
    pub uid: String,
    pub pulls: Vec<Pull>,
}

/// Parses and validates a UIGF v4 file. Records are returned oldest-first.
pub fn import(json: &str) -> Result<Vec<ImportedAccount>, UigfError> {
    let doc: Uigf = serde_json::from_str(json)?;
    if !doc.info.version.starts_with("v4") {
        return Err(UigfError::Version(doc.info.version));
    }
    let mut out = Vec::new();
    for game in Game::ALL {
        for sec in doc.sections(game) {
            let uid = match &sec.uid {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let mut pulls = Vec::with_capacity(sec.list.len());
            for r in &sec.list {
                let raw = r.uigf_gacha_type.as_deref().filter(|_| game == Game::Genshin).unwrap_or(&r.gacha_type);
                let banner: BannerKind = BannerKind::from_gacha_type(game, raw)
                    .ok_or_else(|| UigfError::GachaType { id: r.id.clone(), gacha_type: r.gacha_type.clone() })?;
                pulls.push(Pull {
                    id: r.id.clone(),
                    time: r.time.clone(),
                    name: r.name.clone(),
                    item_type: r.item_type.clone(),
                    rarity: r.rank_type.parse().unwrap_or(3),
                    banner,
                    featured: None,
                });
            }
            // Ids are monotonically increasing snowflakes; compare numerically when possible.
            pulls.sort_by(|a, b| match (a.id.parse::<u64>(), b.id.parse::<u64>()) {
                (Ok(x), Ok(y)) => x.cmp(&y),
                _ => a.time.cmp(&b.time).then_with(|| a.id.cmp(&b.id)),
            });
            out.push(ImportedAccount { game, uid, pulls });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> (Vec<Account>, BTreeMap<Game, Vec<Pull>>) {
        let acc = Account { game: Game::Genshin, uid: "712345678".into(), region: "os_euro".into(), timezone: 1, lang: "en-us".into() };
        let pulls = vec![
            Pull { id: "1730000000000000001".into(), time: "2026-10-01 10:00:00".into(), name: "Furina".into(), item_type: "Character".into(), rarity: 5, banner: BannerKind::Character, featured: None },
            Pull { id: "1730000000000000002".into(), time: "2026-10-01 10:00:00".into(), name: "Favonius Sword".into(), item_type: "Weapon".into(), rarity: 4, banner: BannerKind::Weapon, featured: None },
        ];
        let mut h = BTreeMap::new();
        h.insert(Game::Genshin, pulls);
        (vec![acc], h)
    }

    #[test]
    fn round_trip() {
        let (acc, h) = sample();
        let json = to_json(&export(&acc, &h, 1_762_560_000));
        assert!(json.contains("\"version\": \"v4.0\""));
        assert!(json.contains("\"uigf_gacha_type\": \"301\""));
        let back = import(&json).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].uid, "712345678");
        assert_eq!(back[0].pulls.len(), 2);
        assert_eq!(back[0].pulls[0].name, "Furina");
        assert_eq!(back[0].pulls[1].banner, BannerKind::Weapon);
    }

    #[test]
    fn accepts_numeric_uid_and_timestamp() {
        let json = r#"{"info":{"export_timestamp":"1762560000","export_app":"x","export_app_version":"1","version":"v4.1"},
            "hkrpg":[{"uid":800123456,"timezone":8,"lang":"en-us","list":[
              {"gacha_id":"2001","gacha_type":"11","item_id":"1205","count":"1","time":"2026-09-30 21:00:00","name":"Blade","item_type":"Character","rank_type":"5","id":"1695000000000000002"},
              {"gacha_id":"2001","gacha_type":"11","item_id":"20001","count":"1","time":"2026-09-30 20:59:00","name":"Arrows","item_type":"Light Cone","rank_type":"3","id":"1695000000000000001"}]}]}"#;
        let back = import(json).unwrap();
        assert_eq!(back[0].game, Game::StarRail);
        assert_eq!(back[0].uid, "800123456");
        assert_eq!(back[0].pulls[0].name, "Arrows", "sorted oldest first");
    }

    #[test]
    fn rejects_old_versions_and_bad_types() {
        let v3 = r#"{"info":{"export_timestamp":1,"export_app":"x","export_app_version":"1","version":"v3.0"}}"#;
        assert!(matches!(import(v3), Err(UigfError::Version(_))));
        let bad = r#"{"info":{"export_timestamp":1,"export_app":"x","export_app_version":"1","version":"v4.0"},
            "nap":[{"uid":"1","list":[{"gacha_type":"77","time":"t","rank_type":"4","id":"9"}]}]}"#;
        assert!(matches!(import(bad), Err(UigfError::GachaType { .. })));
        assert!(matches!(import("{"), Err(UigfError::Parse(_))));
    }
}
