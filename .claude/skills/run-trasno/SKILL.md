---
name: run-trasno
description: Build, launch and drive the Trasno Tauri desktop app on a headless Linux container. Use when asked to start Trasno, take a screenshot of it, build it, or click through its UI to verify a change.
---

Trasno is a **Tauri** desktop app (Rust + webkit2gtk on Linux), not
Electron — there is no Chrome DevTools Protocol to attach Playwright to.
Driving it means controlling the real X11 window: Xvfb for a display,
`xdotool` for clicks/keys, `scrot` for screenshots. The REPL driver at
`driver.sh` wraps all of that.

## Prerequisites (one-time per container)

```bash
apt-get update
apt-get install -y webkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev \
  libayatana-appindicator3-dev libsoup-3.0-dev build-essential \
  xvfb scrot xdotool x11-utils
```

## Build

Building with the `custom-protocol` feature makes the binary load the
already-built `dist/` folder directly, so you don't need a live Vite dev
server running alongside it (that's what `cargo tauri dev` would need).

```bash
npm install        # first time only
npm run build       # dist/
cargo build -p trasno --features custom-protocol
```

## Run (agent path)

```bash
tmux new-session -d -s trasno -x 1280 -y 800
tmux send-keys -t trasno '.claude/skills/run-trasno/driver.sh' Enter
sleep 1
tmux send-keys -t trasno 'launch' Enter
sleep 8
tmux send-keys -t trasno 'ss inicio' Enter
sleep 1
tmux capture-pane -t trasno -p
```

Then actually look at `/tmp/shots/inicio.png` (Read tool). A blank or
black image means the window didn't draw yet — wait longer, it's a real
webkit2gtk process, not a headless render.

### Commands

| command | what it does |
|---|---|
| `build` | `npm run build` + `cargo build --features custom-protocol` |
| `launch` | start Xvfb if needed, launch the built binary |
| `ss [name]` | screenshot → `/tmp/shots/<name>.png` (override dir with `SHOT_DIR`) |
| `click X Y` | left-click at screen coordinates (window is at 0,0, ~1180×760) |
| `type TEXT` | type literal text (e.g. a file path) |
| `key KEY` | send a key or chord, e.g. `Return`, `ctrl+l` |
| `opendlg PATH` | for an already-open GTK file/save dialog: `Ctrl+L`, type `PATH`, `Return` — jumps straight to that path instead of clicking through folders |
| `quit` | kill the app (leaves Xvfb running) |

## Clicking without a DOM

There's no `click-text` / `eval` here — webkit2gtk isn't exposing a
DevTools protocol we can reach. To click something:

1. `ss` a screenshot, Read it, find the pixel coordinates of the button.
2. `click X Y` at those coordinates.

Coordinates shift whenever the layout changes, so re-screenshot after
any UI edit before trusting old coordinates. As of this writing, at
1280×800 with the app's default window size: "Abrir conversor" on the
home screen is around (1066, 448); the home button grid starts around
y≈220-320.

## Native file dialogs (open/save)

Trasno uses `tauri-plugin-dialog`, which opens a real GTK file chooser —
not a web `<input type=file>`. Clicking through folders is slow and
fragile; instead, once the dialog is on screen, use `opendlg <path>` (or
manually `key ctrl+l`, `type <path>`, `key Return`) to jump straight to
an absolute path. This works for both "Open" and "Save" dialogs.

## Gotchas

- **Xvfb/tmux/background processes do not survive between agent turns**
  in this environment — each new turn can start with a clean container
  state. Don't assume `launch` from a previous turn is still running;
  check `pgrep -f target/debug/trasno` / `tmux ls` and relaunch if empty.
- **`cargo build` without `--features custom-protocol`** makes the
  binary expect a live dev server at `devUrl` (`http://localhost:1420`)
  and it will show a blank/error window. Always build with that feature
  for the driver flow.
- **Rebuilding after a frontend-only change** still needs both `npm run
  build` (regenerates `dist/`) and `cargo build --features
  custom-protocol` (the frontend gets embedded into the binary via
  `tauri-build`, it doesn't read `dist/` at runtime).
- **No accessible/DOM automation available.** If real click-driven E2E
  testing becomes important, the next step would be enabling Tauri's
  WebKit inspector and driving it over the WebKit remote inspector
  protocol, or adding `data-testid` hooks read via a custom Tauri
  command — neither is set up yet.

## Troubleshooting

- **Blank/black screenshot:** window not drawn yet — `sleep` longer
  after `launch`, or check `/tmp/trasno-driver-app.log` for a crash.
- **"Missing X server" / xdotool errors:** Xvfb isn't running — `launch`
  starts it automatically, but check `pgrep Xvfb` and `/tmp/xvfb.log`.
- **Stale lock:** `rm -f /tmp/.X99-lock; pkill Xvfb` then relaunch.
