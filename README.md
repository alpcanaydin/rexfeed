<div align="center">
  <img src="assets/logo.svg" width="96" alt="rexfeed logo">

<img src="assets/wordmark.svg" width="480" alt="rexfeed">

  *Something worth a glance while your coding agent works, in a side pane of [Rex](https://rex.superlogical.com). For Claude Code, Codex, OpenCode and Pi.*

  [![CI](https://img.shields.io/github/actions/workflow/status/alpcanaydin/rexfeed/ci.yml?style=flat-square&label=CI)](https://github.com/alpcanaydin/rexfeed/actions/workflows/ci.yml)
  [![Release](https://img.shields.io/github/v/release/alpcanaydin/rexfeed?style=flat-square)](https://github.com/alpcanaydin/rexfeed/releases)
  ![macOS on Apple silicon](https://img.shields.io/badge/macOS-Apple%20silicon-555?style=flat-square&logo=apple)
  [![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)

  [How it works](#how-it-works) • [Install](#install) • [Usage](#usage) • [Sources](#sources) • [Video](#how-video-works) • [Config](#config) • [FAQ](#faq)
</div>

https://github.com/user-attachments/assets/6259f35f-b59b-457e-b918-fced48eede0c

<p align="center"><sub>Three Claude Code turns, sped up 4×: the pane opens as the agent starts and closes when it's done. No player? <a href="assets/demo.mp4">Download the video</a>.</sub></p>

While your agent thinks, you wait. rexfeed fills that wait: a pane opens beside the agent and rotates
short tech cards (news, tips, product launches, YouTube and Reddit videos). Images and videos play
right in the terminal through the kitty graphics protocol. When the agent finishes, the pane pauses, so
your attention goes back to the agent.

> [!NOTE]
> rexfeed is an unofficial add-on, not made by or affiliated with Superlogical. It uses Rex's own
> `rex` CLI to open its pane and doesn't modify Rex.

## How it works

rexfeed is opt-in per session. Turn it on in an agent session inside Rex; from then on the agent's
hooks drive the pane:

| Agent | How it hooks in | Opt-in |
|---|---|---|
| Claude Code | `UserPromptSubmit` / `Stop` / `SessionEnd` hooks, plus `SubagentStart` / `SubagentStop` so the pane stays open until running subagents finish | `/rexfeed-start`, `/rexfeed-stop` |
| Codex | the same hooks in `~/.codex/hooks.json` (trust them once with `/hooks`) | `!rexfeed on`, `!rexfeed off` |
| OpenCode | server plugin + TUI plugin | `/rexfeed-start`, `/rexfeed-stop` (or `!rexfeed on`) |
| Pi | extension (`before_agent_start`, `agent_settled`, `session_shutdown`) | `/rexfeed-start`, `/rexfeed-stop` |


| Event | Pane |
|---|---|
| `/rexfeed-start` | Turns rexfeed on for this session; no pane yet |
| Prompt sent (the agent starts working) | Opens the pane |
| Agent finishes | Closes the pane; the next prompt opens it again with unseen cards |
| Pane has focus | Pauses so you can read; resumes when focus leaves |
| 30 cards shown | Loads the next batch of unseen cards |
| No unseen cards left | Starts a background refresh and closes |
| `/rexfeed-stop` (`rexfeed off`) | Closes and turns off for this session |
| Session ends (Claude Code, Codex, Pi) | Closes |

Without opting in the hooks do nothing, so other sessions stay untouched. State is kept per
Rex pane in `~/.cache/rexfeed/state/`. Claude Code, Codex and Pi run inside their pane, so the hooks
read the pane id from `$REX_BLOCK`. OpenCode runs plugins in one shared background server, so its TUI
plugin records which pane shows each session (`~/.cache/rexfeed/opencode/<session>`) and the server
plugin passes that pane as `rexfeed hook … --block <pane>`. The pane is a `rex split` beside the agent's
pane, opened without taking focus.

Cards are picked round-robin across sources, so one feed doesn't take over, and a card you've seen
doesn't come back. When the cache is older than `refresh_minutes`, a prompt starts a fetch in the
background.

## Install

You need a Mac with Apple silicon, [Rex](https://rex.superlogical.com), at least one of Claude Code,
Codex, OpenCode or Pi, [Homebrew](https://brew.sh) and Python 3 (the installer edits the agents' JSON
config with it).
For images and videos: `ffmpeg` and `yt-dlp` (Homebrew installs them for you).

With Homebrew:

```sh
brew trust --tap alpcanaydin/rexfeed   # Homebrew asks you to trust third-party taps
brew install alpcanaydin/rexfeed/rexfeed
rexfeed-setup
```

`rexfeed-setup` hooks rexfeed into your agents. Upgrades need nothing else: the hooks point at
Homebrew's stable `opt` path.

From a release: download `rexfeed-<version>-aarch64-apple-darwin.tar.gz` from
[Releases](https://github.com/alpcanaydin/rexfeed/releases), unpack it and run `./install.sh` inside.
Keep the folder where it is: `~/.local/bin/rexfeed` links to the binary in it.

From a checkout (needs Rust):

```sh
git clone https://github.com/alpcanaydin/rexfeed && cd rexfeed
./install.sh
brew install ffmpeg yt-dlp   # if you don't have them
```

<details>
<summary>What the installer does</summary>

- Builds the binary (from a checkout) and links `~/.local/bin/rexfeed` to it.
- Tells you if `ffmpeg` or `yt-dlp` is missing.
- Runs `scripts/install-hooks.sh`, which hooks into each agent it finds (command on `PATH` or config
  dir present). JSON files are backed up to `*.bak-rexfeed` first; running it again doesn't
  duplicate anything. Config dirs follow `CLAUDE_CONFIG_DIR`, `CODEX_HOME`, `OPENCODE_CONFIG_DIR`
  (or `XDG_CONFIG_HOME`) and `PI_CODING_AGENT_DIR`.
  - Claude Code: `UserPromptSubmit`, `Stop`, `SessionEnd`, `SubagentStart` and `SubagentStop` hooks in `~/.claude/settings.json`;
    `/rexfeed-start` and `/rexfeed-stop` in `~/.claude/commands/`.
  - Codex: the same three hooks in `~/.codex/hooks.json`.
  - OpenCode: `plugins/rexfeed.js` and `plugins/rexfeed-tui/tui.js` in `~/.config/opencode`, the TUI
    plugin listed in `tui.json` (OpenCode 1) and `cli.json` (OpenCode 2), and `/rexfeed-start` and
    `/rexfeed-stop` in `commands/`.
  - Pi: `~/.pi/agent/extensions/rexfeed.ts`, which also registers `/rexfeed-start` and
    `/rexfeed-stop`.
- Migrates an older *techfeed* install: removes its hooks, commands and link, and moves
  `~/.cache/techfeed` to `~/.cache/rexfeed`.
- Runs a first `rexfeed fetch`.
</details>

`~/.local/bin` has to be on your `PATH` for the Claude Code commands and `!rexfeed on` to find
`rexfeed`.

> [!IMPORTANT]
> Codex runs new hooks only after you trust them: start Codex and run `/hooks` once. OpenCode loads
> plugins when its background server starts, so quit OpenCode once after installing (or run
> `pkill -f 'opencode serve'`). Pi picks up the extension on its next start.

## Usage

In an agent session in Rex, run `/rexfeed-start` (Claude Code, OpenCode, Pi) or `!rexfeed on`
(Codex). Send prompts as usual. `/rexfeed-stop` or `!rexfeed off` turns it off again.

| Keys | What |
|---|---|
| `n` `→` `↓` `l` `j` | Next card |
| `p` `←` `↑` `h` `k` | Previous card |
| `space` | Pause / resume |
| `o` / `enter` | Open the link in your browser |
| `q` / `esc` / `ctrl+c` | Close the pane |

The link line is also an OSC 8 hyperlink, so `⌘`-click opens it too.

```
rexfeed fetch                refresh the cache (~/.cache/rexfeed)
rexfeed on | off [--block B] enable or disable for the current Rex pane
rexfeed pane                 run the pane by hand (never pauses for the agent)
rexfeed list                 print cached items
rexfeed hook start|stop|substart|substop|end [--block B]
                             agent hook entry points; the pane defaults to $REX_BLOCK
```

## Sources

- **News:** Hacker News, Show HN, Lobsters, TLDR Tech/AI, The Verge, Ars Technica, TechCrunch, 9to5Mac, Hackaday, Changelog, JavaScript Weekly, This Week in Rust, GitHub Trending, dev.to
- **Reddit:** r/programming, ClaudeAI, LocalLLaMA, commandline, rust, webdev, neovim, selfhosted, MachineLearning (one combined RSS request; Reddit rate-limits hard)
- **Products:** Product Hunt
- **Blogs:** Simon Willison, GitHub, OpenAI, Hugging Face, Vercel, Cloudflare, Rust, Latent Space, Smashing, Josh Comeau
- **Tips:** Simon's TIL, tldr-pages, jbranchaud/til (both cloned locally, refreshed every 3 days)
- **Releases:** Claude Code
- **Video:** Fireship, Beyond Fireship, ThePrimeTime, Theo, Two Minute Papers, Computerphile, Low Level, Linus Tech Tips, Web Dev Simplified, Kurzgesagt

Cards without media get the page's `og:image`. Code in tips is syntax-highlighted.

## How video works

Each refresh downloads the first seconds of the next few unseen videos in the background (YouTube
through `yt-dlp`, Reddit from its `v.redd.it/<id>/HLSPlaylist.m3u8` playlist, both with `ffmpeg`),
and a video card only joins the queue once its clip is on disk. So a video starts the moment its
card shows.

`ffmpeg` decodes the clip into small frames (at most 640px wide). Each frame is zlib-compressed and
sent with the kitty graphics protocol under a new image id, and the previous frame is deleted only
after the new one is placed, so there's no flicker. The terminal scales it to the card. Playback
stops after `video_max` seconds, pauses with the pane, and has no sound. Images are converted with
`ffmpeg` and drawn the same way.

## Config

`~/.config/rexfeed/config.json`, every key optional:

```json
{
  "dwell": 10,
  "video_max": 20,
  "queue_size": 30,
  "refresh_minutes": 45,
  "pane_ratio": 38,
  "idle_close_minutes": 30
}
```

| Key | What |
|---|---|
| `dwell` | Seconds a card without video stays |
| `video_max` | Seconds of a video to play (and to download ahead) |
| `queue_size` | Cards per batch |
| `refresh_minutes` | Cache age that triggers a background fetch |
| `pane_ratio` | Pane size passed to `rex split --ratio` |
| `idle_close_minutes` | Minutes paused (pane focused) before it closes |

To keep rexfeed out of a single session, start it with `REXFEED_OFF=1`.

## FAQ

<details>
<summary>The pane doesn't open</summary>

It only opens inside Rex after you opt in for the session, and only when there are unseen cards.
Codex: run `/hooks` once to trust the hooks. OpenCode: restart it after installing so the plugins load,
and keep the session open in its TUI (the TUI plugin tells rexfeed which pane it's in). Run `rexfeed list` to see the cache and `rexfeed fetch` to fill it. Errors go to
`~/.cache/rexfeed/log.txt`.
</details>

<details>
<summary>Videos don't play</summary>

Check `ffmpeg` and `yt-dlp` are on your `PATH`. YouTube changes often, so `brew upgrade yt-dlp`
fixes most failures.
</details>

<details>
<summary>Does it work outside Rex, or with other agents?</summary>

It works with the agents Rex supports: Claude Code, Codex, OpenCode and Pi. It doesn't work outside
Rex: the pane is opened with `rex split` and keyed on Rex's pane id. `rexfeed pane` runs in any
terminal with kitty graphics, but nothing pauses it.
</details>

<details>
<summary>Does it send anything anywhere?</summary>

No account, no telemetry. It fetches the public feeds above (and linked pages, for their `og:image`) and keeps everything in `~/.cache/rexfeed`.
</details>

## Uninstall

```sh
./uninstall.sh            # hooks, plugins, commands, ~/.local/bin/rexfeed
./uninstall.sh --purge    # also ~/.cache/rexfeed and ~/.config/rexfeed
```

It runs `scripts/install-hooks.sh --remove`, which takes out only rexfeed's own pieces: hook entries
whose command contains `rexfeed hook` (or an old `techfeed hook`) in Claude Code's and Codex's JSON,
backed up first, the OpenCode plugins and their `tui.json` / `cli.json` entries, the Pi extension and
the `/rexfeed-start` / `/rexfeed-stop` commands.

## License

[MIT](LICENSE)
