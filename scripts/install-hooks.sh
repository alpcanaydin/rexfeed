#!/bin/sh
# Hooks rexfeed into each coding agent Rex supports that is installed here:
#
#   Claude Code  ~/.claude/settings.json      UserPromptSubmit / Stop / SessionEnd command hooks
#                ~/.claude/commands/          /rexfeed-start, /rexfeed-stop
#   Codex        ~/.codex/hooks.json          the same hooks (Codex asks you to trust
#                                             new hooks once: run /hooks in Codex)
#   OpenCode     ~/.config/opencode/plugins/rexfeed.js     server plugin (prompt, idle)
#                ~/.config/opencode/plugins/rexfeed-tui/   TUI plugin, listed in cli.json/tui.json:
#                                             tells rexfeed which Rex pane shows a session
#                ~/.config/opencode/commands/ /rexfeed-start, /rexfeed-stop
#   Pi           ~/.pi/agent/extensions/rexfeed.ts  extension (before_agent_start, agent_settled,
#                                             session_shutdown, /rexfeed-start, /rexfeed-stop)
#
# An agent counts as installed when its command is on PATH or its config dir exists.
#
#   install-hooks.sh            add (safe to run again; JSON files are backed up first)
#   install-hooks.sh --remove   take them out again (old techfeed hooks too)
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$HERE/.."
AGENTS="$ROOT/agents"
BIN="$HOME/.local/bin/rexfeed"
MODE=add
[ "${1:-}" = "--remove" ] && MODE=remove

CLAUDE_DIR="${CLAUDE_CONFIG_DIR:-$HOME/.claude}"
CODEX_DIR="${CODEX_HOME:-$HOME/.codex}"
OPENCODE_DIR="${OPENCODE_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/opencode}"
PI_DIR="${PI_CODING_AGENT_DIR:-$HOME/.pi/agent}"

has() { command -v "$1" >/dev/null 2>&1 || [ -d "$2" ]; }

# hooks_json FILE: add or remove the command hooks in a Claude Code style hooks
# file (Claude's settings.json and Codex's hooks.json share the format). Old
# techfeed hooks are dropped either way.
hooks_json() {
  file="$1"
  if [ "$MODE" = remove ] && [ ! -f "$file" ]; then return 0; fi
  mkdir -p "$(dirname "$file")"
  [ -f "$file" ] || echo '{}' > "$file"
  cp "$file" "$file.bak-rexfeed"
  /usr/bin/python3 - "$file" "$BIN" "$MODE" <<'PY'
import json, sys
path, binary, mode = sys.argv[1:4]
raw = open(path).read()
d = json.loads(raw or "{}")
indent = 4 if raw.startswith('{\n    "') else 2
hooks = d.setdefault("hooks", {})
mine = lambda h: "rexfeed hook" in h.get("command", "")
old = lambda h: "techfeed hook" in h.get("command", "")
changed = False
for event in list(hooks):
    groups = []
    for g in hooks[event]:
        kept = [h for h in g.get("hooks", []) if not old(h) and not (mode == "remove" and mine(h))]
        changed |= len(kept) != len(g.get("hooks", []))
        if kept:
            g["hooks"] = kept
            groups.append(g)
    if groups:
        hooks[event] = groups
    else:
        del hooks[event]
if mode != "remove":
    for event, arg in (("UserPromptSubmit", "start"), ("Stop", "stop"), ("SessionEnd", "end")):
        groups = hooks.setdefault(event, [])
        if any(mine(h) for g in groups for h in g.get("hooks", [])):
            continue
        groups.append({"hooks": [{"type": "command", "command": f"{binary} hook {arg}", "timeout": 15}]})
        changed = True
if not hooks:
    del d["hooks"]
open(path, "w").write(json.dumps(d, indent=indent, ensure_ascii=False) + "\n")
print(("removed from" if mode == "remove" else "added to") if changed else "unchanged:", path)
PY
}

# plugin SRC DEST: install a plugin file with the rexfeed path filled in.
plugin() {
  if [ "$MODE" = remove ]; then
    if [ -f "$2" ]; then rm -f "$2" && echo "removed $2"; fi
    return 0
  fi
  mkdir -p "$(dirname "$2")"
  sed "s#__REXFEED__#$BIN#" "$1" > "$2.tmp" && mv -f "$2.tmp" "$2"
  echo "installed $2"
}

# commands SRC DIR [OLD]: copy (or remove) /rexfeed-start and /rexfeed-stop.
commands() {
  for n in rexfeed techfeed; do
    if [ "$MODE" = remove ] || [ "$n" = techfeed ]; then
      for f in "$2/$n-start.md" "$2/$n-stop.md"; do
        if [ -f "$f" ]; then rm -f "$f" && echo "removed $f"; fi
      done
    fi
  done
  [ "$MODE" = remove ] && return 0
  mkdir -p "$2"
  cp "$1/rexfeed-start.md" "$1/rexfeed-stop.md" "$2/"
  echo "installed $2/rexfeed-start.md, rexfeed-stop.md"
}

# tui_list FILE KEY ENTRY: list (or unlist) a TUI plugin in an OpenCode config
# file: tui.json "plugin" (OpenCode 1) or cli.json "plugins" (OpenCode 2).
tui_list() {
  if [ "$MODE" = remove ] && [ ! -f "$1" ]; then return 0; fi
  [ -f "$1" ] || echo '{}' > "$1"
  cp "$1" "$1.bak-rexfeed"
  /usr/bin/python3 - "$1" "$2" "$3" "$MODE" <<'PY'
import json, sys
path, key, entry, mode = sys.argv[1:5]
d = json.loads(open(path).read() or "{}")
plugins = [p for p in d.get(key, []) if "rexfeed-tui" not in (p if isinstance(p, str) else json.dumps(p))]
if mode != "remove":
    plugins.append(entry)
if plugins:
    d[key] = plugins
else:
    d.pop(key, None)
if path.endswith("cli.json") and d and "$schema" not in d:
    d = {"$schema": "https://opencode.ai/v2/cli.json", **d}
open(path, "w").write(json.dumps(d, indent=2) + "\n")
PY
}

found=0
if has claude "$CLAUDE_DIR"; then
  found=1
  hooks_json "$CLAUDE_DIR/settings.json"
  commands "$ROOT/commands" "$CLAUDE_DIR/commands"
fi
if has codex "$CODEX_DIR"; then
  found=1
  hooks_json "$CODEX_DIR/hooks.json"
  if [ "$MODE" = add ]; then echo "    Codex: start it and run /hooks once to trust the three rexfeed hooks."; fi
fi
if has opencode "$OPENCODE_DIR"; then
  found=1
  plugin "$AGENTS/opencode-rexfeed.js" "$OPENCODE_DIR/plugins/rexfeed.js"
  plugin "$AGENTS/opencode-rexfeed-tui.js" "$OPENCODE_DIR/plugins/rexfeed-tui/tui.js"
  if [ "$MODE" = remove ]; then rmdir "$OPENCODE_DIR/plugins/rexfeed-tui" 2>/dev/null || true; fi
  tui_list "$OPENCODE_DIR/tui.json" plugin "file://$OPENCODE_DIR/plugins/rexfeed-tui/tui.js"
  tui_list "$OPENCODE_DIR/cli.json" plugins "file://$OPENCODE_DIR/plugins/rexfeed-tui/tui.js"
  commands "$ROOT/commands/opencode" "$OPENCODE_DIR/commands"
  # OpenCode 2 keeps one background server; it loads new plugins when it restarts.
  if [ "$MODE" = add ]; then echo "    OpenCode: quit OpenCode (or run: pkill -f 'opencode serve') so it loads the plugins."; fi
fi
if has pi "$PI_DIR"; then found=1; plugin "$AGENTS/pi-rexfeed.ts" "$PI_DIR/extensions/rexfeed.ts"; fi
[ "$found" = 1 ] || echo "no supported agent found (Claude Code, Codex, OpenCode, Pi)"
