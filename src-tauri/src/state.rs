//! App state: in-memory history store, persisted as a UIGF file under %APPDATA%\Wishboard.

use crate::models::{Account, Game, Pull};
use crate::{sample, uigf};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Store {
    pub accounts: Vec<Account>,
    pub histories: BTreeMap<Game, Vec<Pull>>,
    /// True while showing generated sample data (nothing imported yet).
    pub is_sample: bool,
}

pub struct AppState {
    pub store: Mutex<Store>,
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir())
        .join("Wishboard")
}

pub fn store_path() -> PathBuf {
    data_dir().join("history.uigf.json")
}

pub fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Merges `incoming` into `existing` by pull id, keeping chronological order. Returns the number of new pulls.
pub fn merge_pulls(existing: &mut Vec<Pull>, incoming: Vec<Pull>) -> usize {
    let mut known: std::collections::HashSet<String> = existing.iter().map(|p| p.id.clone()).collect();
    let before = existing.len();
    for p in incoming {
        if known.insert(p.id.clone()) {
            existing.push(p);
        }
    }
    existing.sort_by(|a, b| a.time.cmp(&b.time).then_with(|| a.id.cmp(&b.id)));
    existing.len() - before
}

impl Store {
    pub fn sample() -> Store {
        Store {
            accounts: Game::ALL.iter().map(|g| sample::account(*g)).collect(),
            histories: Game::ALL.iter().map(|g| (*g, sample::history(*g))).collect(),
            is_sample: true,
        }
    }

    /// Loads the persisted store, falling back to sample data on first run or a broken file.
    pub fn load() -> Store {
        let Ok(json) = fs::read_to_string(store_path()) else { return Store::sample() };
        match uigf::import(&json) {
            Ok(accounts) => {
                let mut store = Store { accounts: Vec::new(), histories: BTreeMap::new(), is_sample: false };
                store.absorb(accounts);
                store
            }
            Err(e) => {
                log::warn!("history file unreadable, using sample data: {e}");
                Store::sample()
            }
        }
    }

    /// Adds imported accounts; replaces sample data on the first real import.
    pub fn absorb(&mut self, imported: Vec<uigf::ImportedAccount>) -> usize {
        if self.is_sample {
            self.accounts.clear();
            self.histories.clear();
            self.is_sample = false;
        }
        let mut added = 0;
        for acc in imported {
            if !self.accounts.iter().any(|a| a.game == acc.game) {
                self.accounts.push(Account { game: acc.game, uid: acc.uid.clone(), region: String::new(), timezone: 8, lang: "en-us".into() });
            }
            added += merge_pulls(self.histories.entry(acc.game).or_default(), acc.pulls);
        }
        added
    }

    pub fn account(&self, game: Game) -> Option<&Account> {
        self.accounts.iter().find(|a| a.game == game)
    }

    pub fn pulls(&self, game: Game) -> &[Pull] {
        self.histories.get(&game).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn to_uigf_json(&self) -> String {
        uigf::to_json(&uigf::export(&self.accounts, &self.histories, unix_now()))
    }

    pub fn save(&self) -> anyhow::Result<PathBuf> {
        let path = store_path();
        fs::create_dir_all(data_dir())?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, self.to_uigf_json())?;
        fs::rename(&tmp, &path)?;
        Ok(path)
    }
}

impl AppState {
    pub fn new() -> AppState {
        AppState { store: Mutex::new(Store::load()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BannerKind;

    fn p(id: &str, time: &str) -> Pull {
        Pull { id: id.into(), time: time.into(), name: "Arrows".into(), item_type: "Light Cone".into(), rarity: 3, banner: BannerKind::Character, featured: None }
    }

    #[test]
    fn merge_dedups_and_sorts() {
        let mut v = vec![p("2", "2026-01-02 00:00:00")];
        let added = merge_pulls(&mut v, vec![p("1", "2026-01-01 00:00:00"), p("2", "2026-01-02 00:00:00"), p("3", "2026-01-03 00:00:00")]);
        assert_eq!(added, 2);
        assert_eq!(v.iter().map(|x| x.id.as_str()).collect::<Vec<_>>(), ["1", "2", "3"]);
    }

    #[test]
    fn first_import_replaces_sample() {
        let mut s = Store::sample();
        let added = s.absorb(vec![uigf::ImportedAccount { game: Game::StarRail, uid: "1".into(), pulls: vec![p("9", "2026-01-01 00:00:00")] }]);
        assert_eq!(added, 1);
        assert!(!s.is_sample);
        assert!(s.pulls(Game::Genshin).is_empty());
        assert_eq!(s.pulls(Game::StarRail).len(), 1);
    }
}
