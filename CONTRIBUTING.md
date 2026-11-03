# Contributing

Thanks for helping. Issues and pull requests are welcome; for anything big,
open an issue first so we can agree on the shape.

## Setup

You need a Mac with Apple silicon, Rex, Claude Code, a recent stable Rust
(edition 2024), and `mpv`, `ffmpeg` and `yt-dlp` for media.

```sh
make build        # target/release/rexfeed
./install.sh      # links ~/.local/bin/rexfeed to it and adds the hooks
make test lint
rexfeed pane      # run the pane by hand in any kitty-graphics terminal
```

After `make build`, the next prompt in a session with `/rexfeed-start` picks up
the new binary.

## Where things are

| Path | What |
|---|---|
| `src/main.rs` | CLI, config, cache, per-pane state, hooks |
| `src/fetch.rs` | Sources, feed parsing, `og:image`, the tip repos |
| `src/pane.rs` | The pane: cards, keys, images, mpv video |
| `commands/` | The `/rexfeed-start` and `/rexfeed-stop` slash commands |
| `scripts/` | Release packaging |

## Adding a source

Keep it free, public and without an account. Reddit and YouTube rate-limit
hard: batch requests, as the existing sources do.

## Style

- Rust: `cargo fmt`, `cargo clippy -- -D warnings`, comments that say why.
- Shell: POSIX `sh`, passes `shellcheck -s sh`.
- Commit messages: imperative, short subject line.
