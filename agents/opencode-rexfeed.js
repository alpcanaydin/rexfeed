// rexfeed for OpenCode: opens, pauses and opts in/out of the rexfeed side pane.
// It runs in OpenCode's server, which serves every pane, so it looks up the
// session's Rex pane in what the TUI half (rexfeed-tui/tui.js) recorded and
// passes it as `--block`. Installed by rexfeed's scripts/install-hooks.sh;
// edits here are overwritten.
import { spawnSync } from "node:child_process"
import { readFileSync } from "node:fs"
import { homedir } from "node:os"
import { join } from "node:path"

const REXFEED = "__REXFEED__"
const DIR = join(homedir(), ".cache", "rexfeed", "opencode")

// The Rex pane showing a session: the TUI's record, else our own $REX_BLOCK
// (OpenCode 1 can run its server inside the pane's process).
function block(sessionID) {
  try {
    if (sessionID && !/[/\\]/.test(sessionID)) {
      const b = readFileSync(join(DIR, sessionID), "utf8").trim()
      if (b) return b
    }
  } catch {}
  return process.env.REX_BLOCK ?? ""
}

// Runs `rexfeed <args> --block <pane>`; returns its output, or "".
// rexfeed never fails a turn, and a missing binary is ignored here too.
function rexfeed(args, sessionID) {
  const b = block(sessionID)
  if (!b) return ""
  try {
    const r = spawnSync(REXFEED, [...args, "--block", b], {
      input: "{}",
      encoding: "utf8",
      timeout: 15000,
    })
    return r.status === 0 ? (r.stdout ?? "").trim() : ""
  } catch {
    return ""
  }
}

// /rexfeed-start and /rexfeed-stop: the commands' text alone can't reach the
// right pane, so the plugin flips the switch for the session.
const OPT = { "rexfeed-start": "on", "rexfeed-stop": "off" }

// OpenCode 2: hooks and events on the plugin context.
async function setup(ctx) {
  if (!ctx || typeof ctx.session?.hook !== "function") return
  await ctx.session.hook("prompt", async (input) => {
    const m = /^\s*\/(rexfeed-(?:start|stop))\b/.exec(input?.prompt?.text ?? "")
    if (m) rexfeed([OPT[m[1]]], input?.sessionID)
    rexfeed(["hook", "start"], input?.sessionID)
  })
  if (typeof ctx.event?.subscribe !== "function") return
  const controller = new AbortController()
  ;(async () => {
    for await (const ev of ctx.event.subscribe({ signal: controller.signal })) {
      if (/^session\.execution\.(succeeded|failed|interrupted)$/.test(ev?.type ?? "")) {
        rexfeed(["hook", "stop"], ev.data?.sessionID)
      }
    }
  })().catch(() => {})
  return async () => controller.abort()
}

// OpenCode 1: chat.message, session.idle, command.execute.before, shell.env.
const server = async () => ({
  "command.execute.before": async (input) => {
    if (OPT[input?.command]) rexfeed([OPT[input.command]], input.sessionID)
  },
  "chat.message": async (input) => {
    rexfeed(["hook", "start"], input?.sessionID)
  },
  event: async ({ event }) => {
    if (event?.type === "session.idle") rexfeed(["hook", "stop"], event.properties?.sessionID)
  },
  // `!rexfeed on` in the prompt runs in the server too: point it at the session's pane.
  "shell.env": async (input, output) => {
    const b = input?.sessionID ? block(input.sessionID) : ""
    if (b && output?.env) output.env.REX_BLOCK = b
  },
})

export default { id: "rexfeed", setup, server }
