//! One-click import: finds the game's own log and web cache on disk and extracts the gacha history URL.
//!
//! Read-only by design: files are opened for reading, never modified, and no account login is involved.
//! The URL contains a temporary `authkey` issued by the game client that the history endpoint accepts.

use crate::models::Game;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// Upper bound for reading a cache file (data_2 is usually 5-40 MB).
const MAX_CACHE_BYTES: u64 = 96 * 1024 * 1024;

#[derive(Debug, Clone, Serialize)]
pub struct Step {
    pub label: String,
    pub detail: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectReport {
    pub game: Game,
    pub steps: Vec<Step>,
    pub log_path: Option<String>,
    pub cache_path: Option<String>,
    /// URL with the authkey masked, safe to display.
    pub url_masked: Option<String>,
    /// Server region parsed from the URL (`region` / `svr_id`).
    pub region: Option<String>,
    pub found: bool,
}

fn user_profile() -> PathBuf {
    std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Users\Default"))
}

/// Candidate log files that reveal the install directory or contain the URL directly.
pub fn candidate_logs(game: Game) -> Vec<PathBuf> {
    let low = user_profile().join("AppData").join("LocalLow");
    match game {
        Game::Genshin => vec![
            low.join(r"miHoYo\Genshin Impact\output_log.txt"),
            low.join(r"miHoYo\Genshin Impact\Player.log"),
            low.join(r"miHoYo\YuanShen\output_log.txt"),
        ],
        Game::StarRail => vec![low.join(r"Cognosphere\Star Rail\Player.log"), low.join(r"miHoYo\StarRail\Player.log")],
        Game::Zenless => vec![low.join(r"miHoYo\ZenlessZoneZero\Player.log")],
        Game::Wuthering => wuwa_install_dirs()
            .into_iter()
            .map(|d| d.join(r"Client\Saved\Logs\Client.log"))
            .collect(),
    }
}

/// Folder names that end the game data path inside the Unity logs.
fn data_dir_markers(game: Game) -> &'static [&'static str] {
    match game {
        Game::Genshin => &["GenshinImpact_Data", "YuanShen_Data"],
        Game::StarRail => &["StarRail_Data"],
        Game::Zenless => &["ZenlessZoneZero_Data"],
        Game::Wuthering => &[],
    }
}

/// Finds `X:/.../<Game>_Data` in a Unity player log. Returns the last occurrence (most recent launch).
pub fn find_data_dir(log: &str, game: Game) -> Option<PathBuf> {
    let mut best: Option<(usize, String)> = None;
    for marker in data_dir_markers(game) {
        for (idx, _) in log.match_indices(marker) {
            let line_start = log[..idx].rfind('\n').map_or(0, |i| i + 1);
            let line = &log[line_start..idx];
            // Find a drive letter like "C:/" or "D:\" closest to the start of the path.
            let bytes = line.as_bytes();
            let drive = (0..bytes.len().saturating_sub(2)).find(|&i| {
                bytes[i].is_ascii_alphabetic()
                    && bytes[i + 1] == b':'
                    && (bytes[i + 2] == b'/' || bytes[i + 2] == b'\\')
                    && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric())
            });
            if let Some(d) = drive {
                let path = format!("{}{}", &line[d..], marker);
                if best.as_ref().map_or(true, |(i, _)| idx > *i) {
                    best = Some((idx, path));
                }
            }
        }
    }
    best.map(|(_, p)| PathBuf::from(p.replace('/', "\\")))
}

/// Parses a dotted version folder name like `2.32.0.0` for ordering.
fn version_key(name: &str) -> Option<Vec<u32>> {
    let parts: Option<Vec<u32>> = name.split('.').map(|s| s.parse().ok()).collect();
    parts.filter(|p| !p.is_empty())
}

/// Picks `webCaches\<highest version>\Cache\Cache_Data\data_2` (falls back to the legacy layout).
pub fn latest_cache_file(data_dir: &Path) -> Option<PathBuf> {
    let web = data_dir.join("webCaches");
    let mut versions: Vec<(Vec<u32>, PathBuf)> = fs::read_dir(&web)
        .ok()?
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            version_key(&name).map(|k| (k, e.path()))
        })
        .collect();
    versions.sort();
    let candidates = versions
        .iter()
        .rev()
        .map(|(_, p)| p.join(r"Cache\Cache_Data\data_2"))
        .chain(std::iter::once(web.join(r"Cache\Cache_Data\data_2")));
    candidates.into_iter().find(|p| p.is_file())
}

fn url_required_markers(game: Game) -> (&'static [&'static str], &'static str) {
    match game {
        Game::Genshin => (&["hk4e", "gacha-v3"], "authkey="),
        Game::StarRail => (&["hkrpg"], "authkey="),
        Game::Zenless => (&["operation-nap", "game_biz=nap"], "authkey="),
        Game::Wuthering => (&["aki/gacha", "gacha/record"], "player_id="),
    }
}

fn is_url_byte(b: u8) -> bool {
    (0x21..=0x7e).contains(&b) && !matches!(b, b'"' | b'\'' | b'<' | b'>' | b'\\' | b'`')
}

/// Scans text or binary cache bytes for gacha history URLs (equivalent to the regex
/// `https://[^\s"'<>\\]+` filtered by game markers). Returns unique URLs in file order.
pub fn extract_gacha_urls(bytes: &[u8], game: Game) -> Vec<String> {
    let (markers, key) = url_required_markers(game);
    let needle = b"https://";
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] != needle {
            i += 1;
            continue;
        }
        let end = bytes[i..].iter().position(|b| !is_url_byte(*b)).map_or(bytes.len(), |n| i + n);
        let url = String::from_utf8_lossy(&bytes[i..end]).to_string();
        i = end.max(i + 1);
        // A cache entry can glue two URLs together; keep the first.
        let url = match url[8..].find("https://") {
            Some(n) => url[..n + 8].to_string(),
            None => url,
        };
        if url.contains(key) && markers.iter().any(|m| url.contains(m)) {
            out.retain(|u| u != &url);
            out.push(url);
        }
    }
    out
}

/// Value of a query parameter (fragment-aware, no percent decoding).
pub fn query_param(url: &str, key: &str) -> Option<String> {
    let query = url.split_once('?')?.1;
    query
        .split(['&', '#'])
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
}

/// Hides the authkey so the URL can be shown in the UI and logs.
pub fn mask_authkey(url: &str) -> String {
    for key in ["authkey=", "player_id="] {
        if let Some(start) = url.find(key).map(|i| i + key.len()) {
            let end = url[start..].find(['&', '#']).map_or(url.len(), |n| start + n);
            let value = &url[start..end];
            let shown: String = value.chars().take(6).collect();
            return format!("{}{}...({} chars){}", &url[..start], shown, value.len(), &url[end..]);
        }
    }
    url.to_string()
}

/// Wuthering Waves has no Unity log; locate the install via uninstall registry entries and common folders.
pub fn wuwa_install_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
        use winreg::RegKey;
        let roots = [
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall"),
            (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
            (HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall"),
        ];
        for (hive, path) in roots {
            let Ok(key) = RegKey::predef(hive).open_subkey_with_flags(path, KEY_READ) else { continue };
            for name in key.enum_keys().flatten() {
                let Ok(app) = key.open_subkey_with_flags(&name, KEY_READ) else { continue };
                let display: String = app.get_value("DisplayName").unwrap_or_default();
                if display.contains("Wuthering Waves") {
                    if let Ok(loc) = app.get_value::<String, _>("InstallLocation") {
                        let base = PathBuf::from(loc);
                        dirs.push(base.join("Wuthering Waves Game"));
                        dirs.push(base);
                    }
                }
            }
        }
    }
    for drive in ["C", "D", "E"] {
        dirs.push(PathBuf::from(format!(r"{drive}:\Wuthering Waves\Wuthering Waves Game")));
        dirs.push(PathBuf::from(format!(r"{drive}:\Program Files\Wuthering Waves\Wuthering Waves Game")));
    }
    dirs.dedup();
    dirs
}

fn read_capped(path: &Path) -> Option<Vec<u8>> {
    let len = fs::metadata(path).ok()?.len();
    if len > MAX_CACHE_BYTES {
        log::warn!("skipping {} ({} bytes)", path.display(), len);
        return None;
    }
    fs::read(path).ok()
}

fn step(label: &str, detail: impl Into<String>, ok: bool) -> Step {
    Step { label: label.to_string(), detail: detail.into(), ok }
}

/// Full detection pipeline used by the Import screen.
pub fn detect(game: Game) -> DetectReport {
    let mut r = DetectReport { game, steps: Vec::new(), log_path: None, cache_path: None, url_masked: None, region: None, found: false };
    let Some(log) = candidate_logs(game).into_iter().find(|p| p.is_file()) else {
        r.steps.push(step("Game log", "Not found. Launch the game once and open the history page in-game.", false));
        return r;
    };
    r.steps.push(step("Game log", log.display().to_string(), true));
    r.log_path = Some(log.display().to_string());

    let bytes = read_capped(&log).unwrap_or_default();
    let mut urls = extract_gacha_urls(&bytes, game);
    if urls.is_empty() && game != Game::Wuthering {
        let text = String::from_utf8_lossy(&bytes);
        match find_data_dir(&text, game) {
            Some(dir) => {
                r.steps.push(step("Game folder", dir.display().to_string(), true));
                match latest_cache_file(&dir) {
                    Some(cache) => {
                        r.steps.push(step("Web cache", cache.display().to_string(), true));
                        r.cache_path = Some(cache.display().to_string());
                        urls = read_capped(&cache).map(|b| extract_gacha_urls(&b, game)).unwrap_or_default();
                    }
                    None => r.steps.push(step("Web cache", "webCaches folder is empty", false)),
                }
            }
            None => r.steps.push(step("Game folder", "Install path not present in the log", false)),
        }
    }
    match urls.last() {
        Some(url) => {
            r.url_masked = Some(mask_authkey(url));
            r.region = query_param(url, "region").or_else(|| query_param(url, "svr_id"));
            r.found = true;
            r.steps.push(step("History URL", "Found, authkey is valid for about 24 hours", true));
        }
        None => r.steps.push(step("History URL", "Open the wish/warp history page in-game, then retry", false)),
    }
    r
}

// TODO(owner): implement the paginated history fetch.
//   - hoyo: GET <api>/getGachaLog?authkey=...&gacha_type=<t>&page=<n>&size=20&end_id=<last id>
//     (hk4e: public-operation-hk4e-sg.hoyoverse.com/gacha_info/api, hkrpg: .../common/gacha_record/api,
//     nap: public-operation-nap-sg.hoyoverse.com/common/gacha_record/api), stop when `list` is empty or
//     an id already stored is reached, sleep ~300 ms between pages, map `retcode -101` to "authkey expired".
//   - wuwa: POST gmserver-api.aki-game2.net/gacha/record/query with playerId/recordId/cardPoolType JSON.
//   - Needs an HTTP client crate (not in the shared dependency set yet); merge results with `state::merge_pulls`.

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "[Subsystems] Discovering subsystems at path C:/Program Files/Genshin Impact/Genshin Impact game/GenshinImpact_Data/UnitySubsystems\n\
        noise\n\
        [Subsystems] Discovering subsystems at path D:/Games/Genshin Impact game/GenshinImpact_Data/UnitySubsystems\n";

    #[test]
    fn finds_latest_data_dir() {
        let p = find_data_dir(LOG, Game::Genshin).unwrap();
        assert_eq!(p, PathBuf::from(r"D:\Games\Genshin Impact game\GenshinImpact_Data"));
        assert!(find_data_dir(LOG, Game::StarRail).is_none());
    }

    #[test]
    fn extracts_urls_from_binary_cache() {
        let mut blob: Vec<u8> = vec![0, 1, 2, 0xff];
        blob.extend_from_slice(b"1/0/https://webstatic.hoyoverse.com/news/index.html\0\0");
        blob.extend_from_slice(b"1/0/https://gs.hoyoverse.com/genshin/event/e20190909gacha-v3/index.html?authkey_ver=1&authkey=OLD%2Bkey&game_biz=hk4e_global#/log\0");
        blob.extend_from_slice(&[0x00, 0x9a, 0x10]);
        blob.extend_from_slice(b"https://public-operation-hk4e-sg.hoyoverse.com/gacha_info/api/getGachaLog?authkey=NEWkey123&lang=en\x00junk");
        let urls = extract_gacha_urls(&blob, Game::Genshin);
        assert_eq!(urls.len(), 2);
        assert!(urls[1].ends_with("lang=en"));
        assert_eq!(query_param(&urls[1], "authkey").as_deref(), Some("NEWkey123"));
        assert!(extract_gacha_urls(&blob, Game::StarRail).is_empty());
    }

    #[test]
    fn wuwa_url_from_client_log() {
        let log = b"[2026.11.02-18.40.11:532][  0]LogKuroGacha: url https://aki-gm-resources-oversea.aki-game.net/aki/gacha/index.html#/record?svr_id=abc&player_id=700123456&lang=en&record_id=f00d\"";
        let urls = extract_gacha_urls(log, Game::Wuthering);
        assert_eq!(urls.len(), 1);
        assert!(urls[0].ends_with("record_id=f00d"));
        assert_eq!(query_param(&urls[0], "player_id").as_deref(), Some("700123456"));
    }

    #[test]
    fn duplicate_urls_keep_latest_position() {
        let blob = b"https://x/hkrpg/getGachaLog?authkey=A https://x/hkrpg/getGachaLog?authkey=B https://x/hkrpg/getGachaLog?authkey=A";
        let urls = extract_gacha_urls(blob, Game::StarRail);
        assert_eq!(urls, vec!["https://x/hkrpg/getGachaLog?authkey=B", "https://x/hkrpg/getGachaLog?authkey=A"]);
    }

    #[test]
    fn masks_authkey() {
        let m = mask_authkey("https://h/getGachaLog?authkey=abcdefghijkl&lang=en");
        assert_eq!(m, "https://h/getGachaLog?authkey=abcdef...(12 chars)&lang=en");
    }

    #[test]
    fn version_ordering() {
        assert!(version_key("2.32.0.0") > version_key("2.9.0.0"));
        assert!(version_key("Cache").is_none());
    }
}
