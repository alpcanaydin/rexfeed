#!/bin/sh
# Build rexfeed (or use the prebuilt binary next to this script), link it into
# ~/.local/bin and hook it into each installed coding agent (Claude Code, Codex,
# OpenCode, Pi) through scripts/install-hooks.sh. Migrates an older "techfeed"
# install on the way.
set -e
cd "$(dirname "$0")"

if [ -f Cargo.toml ]; then
  cargo build --release -q
  bin="$PWD/target/release/rexfeed"
else
  bin="$PWD/rexfeed"   # release tarball or Homebrew
fi
# Homebrew: link the stable opt path, so `brew upgrade` doesn't break the hooks
case "$bin" in
  */Cellar/rexfeed/*/libexec/rexfeed) bin="$(echo "$bin" | sed 's#/Cellar/rexfeed/[^/]*/#/opt/rexfeed/#')" ;;
esac
mkdir -p ~/.local/bin && ln -sf "$bin" ~/.local/bin/rexfeed

# migrate from techfeed (its hooks and commands go in install-hooks.sh)
rm -f ~/.local/bin/techfeed
if [ -d ~/.cache/techfeed ] && [ ! -e ~/.cache/rexfeed ]; then
  mv ~/.cache/techfeed ~/.cache/rexfeed
fi

for t in ffmpeg yt-dlp; do
  command -v "$t" >/dev/null || echo "rexfeed: $t not found; images and videos need it (brew install $t)"
done

scripts/install-hooks.sh
~/.local/bin/rexfeed fetch
