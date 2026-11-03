#!/bin/sh
# Remove rexfeed's agent hooks, plugins, slash commands and binary link (and
# older techfeed ones). --purge also deletes the cache (~/.cache/rexfeed) and
# config (~/.config/rexfeed).
set -e
cd "$(dirname "$0")"
purge=
for a in "$@"; do
  case "$a" in
    --purge) purge=1 ;;
    *) echo "usage: uninstall.sh [--purge]" >&2; exit 2 ;;
  esac
done

scripts/install-hooks.sh --remove
rm -f ~/.local/bin/rexfeed ~/.local/bin/techfeed

if [ -n "$purge" ]; then
  rm -rf ~/.cache/rexfeed ~/.cache/techfeed ~/.config/rexfeed
  echo "cache and config removed"
fi
echo "rexfeed uninstalled"
