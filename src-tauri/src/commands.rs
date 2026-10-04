//! Tauri command layer. Thin wrappers that validate input, lock the store and call the engines.

use crate::models::{BannerKind, Game};
use crate::odds::{self, LuckReport, OddsReport};
use crate::pity::{self, BannerStats, PityRules};
use crate::planner::{self, PlanInput, PlanResult};
use crate::state::AppState;
use crate::{import, uigf};
use serde::Serialize;
use std::collections::HashMap;
use tauri::State;

type CmdResult<T> = Result<T, String>;

fn game(id: &str) -> CmdResult<Game> {
    Game::parse(id).ok_or_else(|| format!("unknown game '{id}'"))
}

fn banner(id: &str) -> CmdResult<BannerKind> {
    BannerKind::parse(id).ok_or_else(|| format!("unknown banner '{id}'"))
}

#[derive(Serialize)]
pub struct GameSummary {
    pub id: &'static str,
    pub title: &'static str,
    pub pull_word: &'static str,
    pub currency: &'static str,
    /// UIGF section key, `None` for games outside the standard.
    pub uigf: Option<&'static str>,
    pub uid: Option<String>,
    pub total: usize,
    pub pity: u32,
}

#[tauri::command]
pub fn list_games(state: State<'_, AppState>) -> CmdResult<Vec<GameSummary>> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    Ok(Game::ALL
        .iter()
        .map(|g| GameSummary {
            id: g.id(),
            title: g.title(),
            pull_word: g.pull_word(),
            currency: g.currency(),
            uigf: g.uigf_key(),
            uid: s.account(*g).map(|a| a.uid.clone()),
            total: s.pulls(*g).len(),
            pity: pity::analyze(*g, BannerKind::Character, s.pulls(*g)).pity,
        })
        .collect())
}

#[derive(Serialize)]
pub struct Overview {
    pub game: Game,
    pub title: &'static str,
    pub uid: Option<String>,
    pub is_sample: bool,
    pub banners: Vec<BannerStats>,
    pub luck: Option<LuckReport>,
    /// P(5-star within the next 10 pulls) per banner id.
    pub next_ten: HashMap<String, f64>,
}

#[tauri::command]
pub fn get_overview(state: State<'_, AppState>, game_id: String) -> CmdResult<Overview> {
    let g = game(&game_id)?;
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let banners = pity::analyze_game(g, s.pulls(g));
    let luck = banners.iter().find(|b| b.banner == BannerKind::Character && b.five_stars > 0).map(|b| {
        odds::luck_percentile(&b.rules, b.avg_pity, b.five_stars, 6000, 0x5eed ^ b.total as u64)
    });
    let next_ten = banners
        .iter()
        .map(|b| (format!("{:?}", b.banner).to_lowercase(), odds::p_within(&b.rules, b.pity, 10)))
        .collect();
    Ok(Overview { game: g, title: g.title(), uid: s.account(g).map(|a| a.uid.clone()), is_sample: s.is_sample, banners, luck, next_ten })
}

#[derive(Serialize)]
pub struct HistoryRow {
    pub id: String,
    pub time: String,
    pub name: String,
    pub item_type: String,
    pub rarity: u8,
    pub banner: BannerKind,
    /// Pulls counted towards this item's rarity pity when it dropped.
    pub pity: u32,
}

#[derive(Serialize)]
pub struct HistoryPage {
    pub total: usize,
    pub rows: Vec<HistoryRow>,
}

#[tauri::command]
pub fn get_history(
    state: State<'_, AppState>,
    game_id: String,
    banner_id: Option<String>,
    min_rarity: Option<u8>,
    query: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> CmdResult<HistoryPage> {
    let g = game(&game_id)?;
    let filter_banner = banner_id.as_deref().map(banner).transpose()?;
    let q = query.unwrap_or_default().to_lowercase();
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let mut counters: HashMap<BannerKind, (u32, u32)> = HashMap::new();
    let mut rows: Vec<HistoryRow> = Vec::new();
    for p in s.pulls(g) {
        let c = counters.entry(p.banner).or_insert((0, 0));
        c.0 += 1;
        c.1 += 1;
        let pity = match p.rarity {
            5 => std::mem::take(&mut c.0),
            4 => std::mem::take(&mut c.1),
            _ => c.0,
        };
        if p.rarity >= 5 {
            c.1 = 0;
        }
        let keep = filter_banner.map_or(true, |b| b == p.banner)
            && p.rarity >= min_rarity.unwrap_or(3)
            && (q.is_empty() || p.name.to_lowercase().contains(&q));
        if keep {
            rows.push(HistoryRow { id: p.id.clone(), time: p.time.clone(), name: p.name.clone(), item_type: p.item_type.clone(), rarity: p.rarity, banner: p.banner, pity });
        }
    }
    rows.reverse(); // newest first
    let total = rows.len();
    let rows = rows.into_iter().skip(offset.unwrap_or(0)).take(limit.unwrap_or(200).min(2000)).collect();
    Ok(HistoryPage { total, rows })
}

#[tauri::command]
pub fn get_odds(game_id: String, banner_id: String, pity: u32, guaranteed: bool, pulls: u32) -> CmdResult<OddsReport> {
    let rules = PityRules::for_banner(game(&game_id)?, banner(&banner_id)?);
    Ok(odds::report(&rules, pity.min(rules.hard_pity - 1), guaranteed, pulls.min(1000)))
}

#[tauri::command]
pub fn planner_preset(state: State<'_, AppState>, game_id: String) -> CmdResult<PlanInput> {
    let g = game(&game_id)?;
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let st = pity::analyze(g, BannerKind::Character, s.pulls(g));
    Ok(PlanInput { pity: st.pity, guaranteed: st.guaranteed, ..PlanInput::preset(g) })
}

#[tauri::command]
pub fn plan_savings(input: PlanInput) -> CmdResult<PlanResult> {
    if input.days > 3650 {
        return Err("target date is too far away".into());
    }
    Ok(planner::plan(&input))
}

#[tauri::command]
pub async fn detect_import(game_id: String) -> CmdResult<import::DetectReport> {
    let g = game(&game_id)?;
    // File scanning can take a second on large caches; keep it off the main thread.
    std::thread::spawn(move || import::detect(g)).join().map_err(|_| "detection thread panicked".to_string())
}

#[tauri::command]
pub fn start_import(game_id: String) -> CmdResult<String> {
    let g = game(&game_id)?;
    // TODO(owner): call the paginated history fetch (see import.rs), stream progress with
    // `app.emit("import-progress", ...)`, then `store.absorb(...)` and `store.save()`.
    Err(format!("{} history download is not wired up yet. Use UIGF import meanwhile.", g.title()))
}

#[derive(Serialize)]
pub struct ExportResult {
    pub path: String,
    pub bytes: usize,
    pub records: usize,
}

#[tauri::command]
pub fn export_uigf(state: State<'_, AppState>, path: Option<String>) -> CmdResult<ExportResult> {
    let s = state.store.lock().map_err(|e| e.to_string())?;
    let json = s.to_uigf_json();
    let target = match path {
        Some(p) => std::path::PathBuf::from(p),
        None => {
            let docs = std::env::var_os("USERPROFILE").map(std::path::PathBuf::from).unwrap_or_default().join("Documents");
            docs.join(format!("Wishboard-UIGF-{}.json", crate::sample::format_time(crate::state::unix_now() as i64)[..10].to_string()))
        }
    };
    std::fs::write(&target, &json).map_err(|e| format!("cannot write {}: {e}", target.display()))?;
    Ok(ExportResult { path: target.display().to_string(), bytes: json.len(), records: s.histories.values().map(|v| v.len()).sum() })
}

#[derive(Serialize)]
pub struct ImportResult {
    pub accounts: usize,
    pub added: usize,
}

#[tauri::command]
pub fn import_uigf(state: State<'_, AppState>, json: String) -> CmdResult<ImportResult> {
    let accounts = uigf::import(&json).map_err(|e| e.to_string())?;
    let n = accounts.len();
    let mut s = state.store.lock().map_err(|e| e.to_string())?;
    let added = s.absorb(accounts);
    s.save().map_err(|e| e.to_string())?;
    Ok(ImportResult { accounts: n, added })
}
