// rexfeed for Pi: opens, pauses and closes the rexfeed side pane with Pi's
// turns. Pi runs inside its Rex pane, so rexfeed finds the pane through
// $REX_BLOCK, which the spawned process inherits. Installed by rexfeed's
// scripts/install-hooks.sh; edits here are overwritten on the next install.
import { spawnSync } from "node:child_process"

const REXFEED = "__REXFEED__"

// Runs `rexfeed <args>`; returns its output, or "" on any failure.
// rexfeed never fails a turn, and a missing binary is ignored here too.
function rexfeed(args: string[]): string {
  try {
    const r = spawnSync(REXFEED, args, { input: "{}", encoding: "utf8", timeout: 15000 })
    return r.status === 0 ? (r.stdout ?? "").trim() : ""
  } catch {
    return ""
  }
}

export default function (pi: any) {
  // A message from you starts a turn: open or resume the pane (if opted in).
  pi.on("before_agent_start", async () => {
    rexfeed(["hook", "start"])
  })
  // Pi is done (retries and queued work included): pause.
  pi.on("agent_settled", async () => {
    rexfeed(["hook", "stop"])
  })
  // Pi exits: close the pane.
  pi.on("session_shutdown", async () => {
    rexfeed(["hook", "end"])
  })
  // Opt in or out for this session, like /rexfeed-start and /rexfeed-stop in Claude Code.
  for (const [name, arg, description] of [
    ["rexfeed-start", "on", "Open the rexfeed side pane for this session"],
    ["rexfeed-stop", "off", "Close the rexfeed side pane and turn it off for this session"],
  ]) {
    pi.registerCommand(name, {
      description,
      handler: async (_args: string, ctx: any) => {
        const out = rexfeed([arg]) || "rexfeed: failed (is ~/.local/bin/rexfeed there?)"
        try {
          ctx.ui.notify(out, "info")
        } catch {}
      },
    })
  }
}
