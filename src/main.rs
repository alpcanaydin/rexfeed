//! rexfeed: short tech news / tips / video side pane for coding agent sessions in Rex.

mod fetch;
mod logo;
mod pane;

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const HELP: &str = "rexfeed: tech news / tips side pane for coding agents in Rex

USAGE:
  rexfeed fetch                refresh the cache from all sources
  rexfeed pane [--key KEY]     run the side pane (the hook opens it)
  rexfeed on | off [--block B] enable/disable for this session (/rexfeed-start, /rexfeed-stop)
  rexfeed hook start|stop|end [--block B]
                               agent hook entry points (Claude Code, Codex, OpenCode, Pi)

  The Rex pane defaults to $REX_BLOCK; --block overrides it.
  rexfeed list                 print cached items
";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub id: String,
    pub source: String,
    pub title: String,
    pub url: String,
    pub body: String,
    pub image: Option<String>,
    pub video: Option<String>,
    pub kind: String,
    pub ts: f64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Store {
    pub fetched: f64,
    pub items: Vec<Item>,
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub dwell: f64,
    pub video_max: u32,
    pub queue_size: usize,
    pub refresh_minutes: f64,
    pub pane_ratio: u32,
    pub idle_close_minutes: f64,
    pub video_muted: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            dwell: 10.0,
            video_max: 20,
            queue_size: 30,
            refresh_minutes: 45.0,
            pane_ratio: 38,
            idle_close_minutes: 30.0,
            video_muted: true,
        }
    }
}

pub fn home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
}
pub fn cache_dir() -> PathBuf {
    let p = home().join(".cache/rexfeed");
    let old = home().join(".cache/techfeed"); // pre-rename installs
    if !p.exists() && old.exists() {
        let _ = std::fs::rename(&old, &p);
    }
    let _ = std::fs::create_dir_all(&p);
    p
}
pub fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

pub fn config() -> Config {
    std::fs::read_to_string(home().join(".config/rexfeed/config.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn log(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(cache_dir().join("log.txt"))
    {
        let _ = writeln!(f, "{:.0} {}", now(), msg);
    }
}

pub fn write_json<T: Serialize>(path: &PathBuf, v: &T) {
    let tmp = path.with_extension("tmp");
    if std::fs::write(&tmp, serde_json::to_vec(v).unwrap()).is_ok() {
        let _ = std::fs::rename(tmp, path);
    }
}

pub fn load_store() -> Store {
    std::fs::read(cache_dir().join("items.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn load_seen() -> std::collections::HashSet<String> {
    std::fs::read(cache_dir().join("seen.json"))
        .ok()
        .and_then(|b| serde_json::from_slice::<Vec<String>>(&b).ok())
        .unwrap_or_default()
        .into_iter()
        .collect()
}

pub fn mark_seen(id: &str) {
    let mut v: Vec<String> = std::fs::read(cache_dir().join("seen.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    if !v.iter().any(|x| x == id) {
        v.push(id.to_string());
    }
    if v.len() > 5000 {
        v.drain(..v.len() - 5000);
    }
    write_json(&cache_dir().join("seen.json"), &v);
}

/// Unseen items, round-robin across sources so cards stay varied.
pub fn build_queue(cfg: &Config) -> Vec<Item> {
    let seen = load_seen();
    let mut by_src: Vec<(String, std::collections::VecDeque<Item>)> = Vec::new();
    // REXFEED_ONLY=video (or tip, news…) narrows the queue; handy for trying one card type
    let only = std::env::var("REXFEED_ONLY").ok();
    for it in load_store()
        .items
        .into_iter()
        .filter(|x| !seen.contains(&x.id))
        // a video card waits until its clip is cached, so it always starts instantly
        .filter(|x| {
            x.video
                .as_deref()
                .is_none_or(|v| crate::fetch::video_cache(v).exists())
        })
        .filter(|x| match only.as_deref() {
            Some("video") => x.video.is_some(),
            Some(k) => x.kind == k,
            None => true,
        })
    {
        match by_src.iter_mut().find(|(s, _)| *s == it.source) {
            Some((_, q)) => q.push_back(it),
            None => by_src.push((it.source.clone(), [it].into())),
        }
    }
    fastrand::shuffle(&mut by_src);
    // REXFEED_LEAD=video: start with a (cached) video card, e.g. for demos
    let lead = (std::env::var("REXFEED_LEAD").as_deref() == Ok("video"))
        .then(|| {
            by_src.iter_mut().find_map(|(_, q)| {
                q.iter()
                    .position(|x| x.video.is_some())
                    .and_then(|i| q.remove(i))
            })
        })
        .flatten();
    let mut out: Vec<Item> = lead.into_iter().collect();
    while out.len() < cfg.queue_size && by_src.iter().any(|(_, q)| !q.is_empty()) {
        for (_, q) in by_src.iter_mut() {
            if out.len() >= cfg.queue_size {
                break;
            }
            if let Some(x) = q.pop_front() {
                out.push(x);
            }
        }
    }
    out
}

// ---------------------------------------------------------------- per-pane state

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PaneState {
    pub status: String, // running | paused | ended
    pub pane_pid: Option<u32>,
    #[serde(default)]
    pub spawned_at: f64,
    #[serde(default)]
    pub enabled: bool, // opt-in per session via /rexfeed-start
}

fn state_path(key: &str) -> PathBuf {
    let d = cache_dir().join("state");
    let _ = std::fs::create_dir_all(&d);
    let safe: String = key
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    d.join(format!("{safe}.json"))
}
pub fn read_state(key: &str) -> PaneState {
    std::fs::read(state_path(key))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
pub fn update_state(key: &str, f: impl FnOnce(&mut PaneState)) {
    let mut s = read_state(key);
    f(&mut s);
    write_json(&state_path(key), &s);
}

fn alive(pid: Option<u32>) -> bool {
    pid.map(|p| unsafe { libc::kill(p as i32, 0) } == 0)
        .unwrap_or(false)
}

/// The Rex pane to act on: `--block BLOCK` if given (OpenCode's shared server
/// has no pane of its own), else `$REX_BLOCK` of the agent's pane.
fn block(args: &[String]) -> Option<String> {
    args.iter()
        .position(|a| a == "--block")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .or_else(|| std::env::var("REX_BLOCK").ok())
        .filter(|b| !b.is_empty())
}

fn hook(action: &str, key: Option<String>) {
    if !matches!(action, "on" | "off") {
        let mut sink = String::new();
        let _ = std::io::stdin().read_to_string(&mut sink); // drain hook JSON
    }
    let Some(key) = key else {
        return;
    };
    if std::env::var("REXFEED_OFF").is_ok() {
        return;
    }
    match action {
        // the agent finished its turn: the pane closes (it reopens on the next prompt)
        "stop" => update_state(&key, |s| {
            s.status = "closed".into();
            s.spawned_at = 0.0;
        }),
        "end" => update_state(&key, |s| s.status = "ended".into()),
        "off" => update_state(&key, |s| {
            s.enabled = false;
            s.status = "ended".into();
        }),
        "on" | "start" => {
            if action == "on" {
                // opting in doesn't open the pane; the next prompt (agent starts working) does
                update_state(&key, |s| s.enabled = true);
                return;
            }
            if !read_state(&key).enabled {
                return;
            }
            update_state(&key, |s| s.status = "running".into());
            let cfg = config();
            let exe = std::env::current_exe().unwrap();
            if now() - load_store().fetched > cfg.refresh_minutes * 60.0 {
                let _ = Command::new(&exe)
                    .arg("fetch")
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .process_group(0)
                    .spawn();
            }
            let st = read_state(&key);
            // spawned_at covers the gap before the new pane registers its pid
            if alive(st.pane_pid) || now() - st.spawned_at < 10.0 || build_queue(&cfg).is_empty() {
                return;
            }
            update_state(&key, |s| s.spawned_at = now());
            let r = Command::new("rex")
                .args([
                    "split",
                    "--focus=false",
                    "--ratio",
                    &cfg.pane_ratio.to_string(),
                    "-b",
                    &key,
                    "--",
                ])
                // `rex split` doesn't inherit our env: hand REXFEED_LEAD/REXFEED_ONLY over explicitly
                .arg("/usr/bin/env")
                .args(
                    ["REXFEED_LEAD", "REXFEED_ONLY"]
                        .iter()
                        .filter_map(|k| std::env::var(k).ok().map(|v| format!("{k}={v}"))),
                )
                .arg(&exe)
                .args(["pane", "--key", &key])
                .output();
            match r {
                Ok(o) if !o.status.success() => log(&format!(
                    "split fail: {}",
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => log(&format!("split fail: {e}")),
                _ => {}
            }
        }
        _ => {}
    }
}

use std::os::unix::process::CommandExt;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("fetch") => fetch::fetch_all(),
        Some("pane") => {
            let key = args
                .iter()
                .position(|a| a == "--key")
                .and_then(|i| args.get(i + 1))
                .cloned();
            let key = key.unwrap_or_else(|| {
                update_state("manual", |s| s.status = "running".into());
                "manual".into()
            });
            pane::run(&key);
        }
        Some("hook") => hook(args.get(1).map(|s| s.as_str()).unwrap_or(""), block(&args)),
        Some(a @ ("on" | "off")) => {
            let Some(key) = block(&args) else {
                println!("rexfeed: not inside a Rex pane (REX_BLOCK unset, no --block)");
                return;
            };
            hook(a, Some(key));
            println!(
                "rexfeed {}",
                if a == "on" {
                    "on for this session"
                } else {
                    "off for this session"
                }
            );
        }
        Some("list") => {
            for x in load_store().items {
                println!(
                    "{:<16} {}{}{}",
                    x.source.chars().take(16).collect::<String>(),
                    if x.image.is_some() { "🖼 " } else { "" },
                    if x.video.is_some() { "▶ " } else { "" },
                    x.title
                );
            }
        }
        _ => print!("{HELP}"),
    }
}
