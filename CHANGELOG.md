# Changelog

All notable changes to rexfeed. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.1] - 2026-11-04

### Changed

- The pane stays open while subagents are still running: Claude Code's new `SubagentStart` / `SubagentStop` hooks (`rexfeed hook substart|substop`) open it when a subagent starts and close it only once the turn has ended and no subagent is left. A subagent that never reports back stops counting after 3 hours. Run `rexfeed-setup` (or `scripts/install-hooks.sh`) again to add the new hooks.

## [0.1.0] - 2026-11-03

### Added

- A side pane in Rex that rotates short tech cards beside a coding agent session (Claude Code, Codex, OpenCode or Pi): news, tips, product launches, releases and videos.
- Opt-in per session with `/rexfeed-start` and `/rexfeed-stop` (Claude Code, OpenCode, Pi) or `!rexfeed on` / `!rexfeed off` (Codex).
- Hooks for every agent Rex supports, installed by `scripts/install-hooks.sh` for the agents it finds: `UserPromptSubmit`, `Stop` and `SessionEnd` hooks for Claude Code and Codex, a server + TUI plugin for OpenCode 1 and 2, and an extension for Pi. They open, pause and close the pane.
- `--block BLOCK` on `rexfeed hook` and `rexfeed on|off` to name the Rex pane (default `$REX_BLOCK`); the OpenCode plugin uses it.
- Pauses while the agent is idle or the pane has focus, closes after `idle_close_minutes` paused, and refreshes in the background when the cache is stale.
- Inline images (kitty graphics, `og:image` fallback) and inline video through `mpv --vo=kitty`, with `yt-dlp` for YouTube and HLS for Reddit.
- Sources: Hacker News, Lobsters, Reddit, Product Hunt, tech news sites and blogs, TIL and tldr-pages tips, Claude Code releases and YouTube channels.
- Keys for next/previous, pause, open link, mute and close; the link is also an OSC 8 hyperlink.
- `~/.config/rexfeed/config.json` for timing, batch size, pane size and sound.
- `install.sh` (migrates an older techfeed install) and `uninstall.sh`, both through `scripts/install-hooks.sh`.

[0.1.1]: https://github.com/alpcanaydin/rexfeed/releases/tag/v0.1.1
[0.1.0]: https://github.com/alpcanaydin/rexfeed/releases/tag/v0.1.0
