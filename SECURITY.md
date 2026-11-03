# Security policy

rexfeed adds hooks to Claude Code, downloads content from public feeds and
plays it in your terminal through `mpv`, `ffmpeg` and `yt-dlp`. Problems in any
of that are worth reporting.

## Reporting a vulnerability

Please don't open a public issue. Report it privately through
[GitHub's security advisories](https://github.com/alpcanaydin/rexfeed/security/advisories/new)
with what you found, how to reproduce it and which versions are affected. You'll
get an answer within a few days, and a fix and an advisory when it's confirmed.

## Supported versions

Fixes go into the latest release. Update by installing the new release (or
`git pull && ./install.sh` in a checkout).

## What rexfeed touches

- `~/.claude/settings.json`: `UserPromptSubmit`, `Stop` and `SessionEnd` hooks
  that run `~/.local/bin/rexfeed hook …`. The file is backed up to
  `settings.json.bak-rexfeed` before it changes.
- `~/.claude/commands/rexfeed-start.md` and `rexfeed-stop.md`.
- `~/.local/bin/rexfeed`, a link to the binary.
- `~/.cache/rexfeed`: the item cache, per-pane state, downloaded media, cloned
  tip repositories and a log. `~/.config/rexfeed/config.json` if you create it.
- Feed text and titles are drawn in your terminal; control characters in them
  that reach the terminal are worth reporting.
