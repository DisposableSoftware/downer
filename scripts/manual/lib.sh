# Helpers for driving Downer on Hyprland. Source from a script run in the repo root.
# See "Manual testing on Hyprland" in CLAUDE.md for why each guard exists.
set -u

APP=io.github.DisposableSoftware.Downer
BIN=${BIN:-./result/bin/downer}

wins() { hyprctl clients -j | jq "[.[] | select(.class==\"$APP\")]"; }
win_count() { wins | jq length; }
active() { hyprctl activewindow -j | jq -r "$1"; }

# The app is single-instance: launching while the user has Downer open would join
# their process, and killing it would close their windows.
require_no_instance() {
  if [ "$(win_count)" != 0 ]; then
    echo "Downer is already running; close it first (refusing to touch the user's windows)" >&2
    exit 2
  fi
  if pidof hyprlock >/dev/null; then echo "screen is locked" >&2; exit 2; fi
}

# launch [PATH] [XDG_CONFIG_HOME]: starts Downer (on PATH, if given) and focuses it. Sets PID.
launch() {
  local dir=${1:-} cfg=${2:-}
  [ -x "$BIN" ] || { echo "run nix build first" >&2; exit 2; }
  if [ -n "$cfg" ]; then XDG_CONFIG_HOME=$cfg "$BIN" ${dir:+"$dir"} >"${LOG:-/dev/null}" 2>&1 &
  else "$BIN" ${dir:+"$dir"} >"${LOG:-/dev/null}" 2>&1 & fi
  for _ in $(seq 25); do [ "$(win_count)" -gt 0 ] && break; sleep 0.2; done
  sleep 0.4
  PID=$(wins | jq -r '.[0].pid')
  hyprctl dispatch "hl.dsp.focus({ window = \"class:$APP\" })" >/dev/null
  sleep 0.3
}

# keys WTYPE-ARGS...: sends keys only if a Downer window (main or dialog) is focused.
keys() {
  if [ "$(active .class)" != "$APP" ]; then
    echo "Downer lost focus; stopping so keys don't go to another app" >&2
    cleanup; exit 3
  fi
  # Without a per-key delay wtype sends a whole sequence instantly, faster than the
  # list view processes focus moves; real typing is never that fast.
  wtype -d 100 "$@"; sleep 0.6
}

shot() { grim -g "$(active '"\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')" "$1"; }

cleanup() { [ -n "${PID:-}" ] && kill "$PID" 2>/dev/null; true; }

FAILS=0
check() { # check DESCRIPTION EXPECTED ACTUAL
  if [ "$2" = "$3" ]; then echo "PASS  $1"; else echo "FAIL  $1 (expected '$2', got '$3')"; FAILS=$((FAILS+1)); fi
}
