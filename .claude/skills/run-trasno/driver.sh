#!/usr/bin/env bash
# REPL driver para Trasno (app Tauri/webkit2gtk) en Linux sen pantalla.
#
# Tauri non é Electron: non hai Chrome DevTools Protocol, así que
# Playwright non serve para automatizala. Este driver controla a xanela
# real (Xvfb + xdotool) por coordenadas de pantalla e capturas (scrot),
# non por selectores DOM. Pensado para tmux: unha liña de stdin = un
# comando.
#
# Comandos:
#   build              cargo build (con custom-protocol) + npm run build
#   launch             arrinca Xvfb (se non existe) e lanza o binario
#   ss [nome]          captura de pantalla -> $SHOT_DIR/<nome>.png
#   click X Y          clic esquerdo en (X, Y) da xanela
#   type TEXTO         escribe TEXTO (delay entre teclas)
#   key TECLA          preme unha tecla/combinación (ex: Return, ctrl+l)
#   opendlg RUTA       abre un diálogo GTK xa aberto: Ctrl+L, escribe RUTA, Return
#   quit               mata a app (o driver segue aceso; 'launch' relánzaa)
#   exit               mata a app e sae do driver (non toca o Xvfb)
#
# Uso típico (dende a raíz do repo):
#   tmux new-session -d -s trasno -x 1280 -y 800
#   tmux send-keys -t trasno '.claude/skills/run-trasno/driver.sh' Enter
#   tmux send-keys -t trasno 'build' Enter      # só a primeira vez / tras cambiar código
#   tmux send-keys -t trasno 'launch' Enter
#   tmux send-keys -t trasno 'ss inicio' Enter
#   tmux capture-pane -t trasno -p

set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
BIN_PATH="$REPO_ROOT/target/debug/trasno"
SHOT_DIR="${SHOT_DIR:-/tmp/shots}"
DISPLAY_NUM="${TRASNO_DISPLAY:-:99}"
APP_LOG="/tmp/trasno-driver-app.log"

mkdir -p "$SHOT_DIR"

APP_PID=""

ensure_xvfb() {
  if ! pgrep -f "Xvfb $DISPLAY_NUM" >/dev/null 2>&1; then
    echo "arrincando Xvfb en $DISPLAY_NUM..."
    rm -f "/tmp/.X${DISPLAY_NUM#:}-lock"
    Xvfb "$DISPLAY_NUM" -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 &
    disown
    sleep 1.5
  fi
}

cmd_build() {
  echo "compilando frontend (vite build)..."
  (cd "$REPO_ROOT" && npm run build) || { echo "ERRO: fallou npm run build"; return; }
  echo "compilando backend (cargo build --features custom-protocol)..."
  (cd "$REPO_ROOT" && cargo build -p trasno --features custom-protocol) || { echo "ERRO: fallou cargo build"; return; }
  echo "build ok: $BIN_PATH"
}

cmd_launch() {
  if [ -n "$APP_PID" ] && kill -0 "$APP_PID" 2>/dev/null; then
    echo "xa está lanzada (pid $APP_PID)"
    return
  fi
  if [ ! -x "$BIN_PATH" ]; then
    echo "ERRO: non existe $BIN_PATH. Executa 'build' primeiro."
    return
  fi
  ensure_xvfb
  DISPLAY="$DISPLAY_NUM" GDK_BACKEND=x11 WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    "$BIN_PATH" >"$APP_LOG" 2>&1 &
  APP_PID=$!
  disown
  echo "lanzada, pid=$APP_PID. agardando a que debuxe..."
  sleep 6
  echo "lista (revisa con 'ss' se queres confirmar)"
}

cmd_ss() {
  local name="${1:-ss-$(date +%s)}"
  local f="$SHOT_DIR/$name.png"
  DISPLAY="$DISPLAY_NUM" scrot -o "$f" 2>&1
  echo "screenshot: $f"
}

cmd_click() {
  local x="$1" y="$2"
  DISPLAY="$DISPLAY_NUM" xdotool mousemove "$x" "$y" click 1
  echo "click $x $y"
}

cmd_type() {
  DISPLAY="$DISPLAY_NUM" xdotool type --delay 30 -- "$*"
  echo "type: $*"
}

cmd_key() {
  DISPLAY="$DISPLAY_NUM" xdotool key "$1"
  echo "key: $1"
}

cmd_opendlg() {
  local ruta="$1"
  DISPLAY="$DISPLAY_NUM" xdotool key ctrl+l
  sleep 0.4
  DISPLAY="$DISPLAY_NUM" xdotool type --delay 30 -- "$ruta"
  sleep 0.2
  DISPLAY="$DISPLAY_NUM" xdotool key Return
  echo "opendlg: $ruta"
}

cmd_quit() {
  [ -n "$APP_PID" ] && kill "$APP_PID" 2>/dev/null
  pkill -f "$BIN_PATH" 2>/dev/null
  APP_PID=""
  echo "app pechada (o driver segue aceso; 'launch' relánzaa, 'exit' pecha o driver)"
}

echo "driver de Trasno — 'help' para ver comandos"
while IFS= read -r line; do
  set -- $line
  cmd="${1:-}"
  shift || true
  case "$cmd" in
    build) cmd_build ;;
    launch) cmd_launch ;;
    ss) cmd_ss "$@" ;;
    click) cmd_click "$@" ;;
    type) cmd_type "$@" ;;
    key) cmd_key "$@" ;;
    opendlg) cmd_opendlg "$@" ;;
    quit) cmd_quit ;;
    exit) cmd_quit; break ;;
    help) echo "comandos: build launch ss click type key opendlg quit exit" ;;
    "") ;;
    *) echo "descoñecido: $cmd (proba 'help')" ;;
  esac
  echo "driver> "
done
