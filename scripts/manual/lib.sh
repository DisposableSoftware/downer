# Helpers for driving Downer. Source from a script run in the repo root.
# See "Manual testing on Hyprland" in CLAUDE.md for why each guard exists. Under
# scripts/manual/headless.sh (DOWNER_HEADLESS set) the same helpers drive a headless
# sway instead, which works with the screen locked and never touches the user's windows.
set -u

APP=io.github.DisposableSoftware.Downer
BIN=${BIN:-./result/bin/downer}

if [ -n "${DOWNER_HEADLESS:-}" ]; then
  # Same fields as hyprctl clients -j: class, pid, title, at, size, focused.
  all_wins() {
    swaymsg -t get_tree | jq "[recurse(.nodes[]?, .floating_nodes[]?) | select(.app_id != null)
      | {class: .app_id, pid, title: .name, at: [.rect.x, .rect.y],
         size: [.rect.width, .rect.height], focused}]"
  }
  active() { all_wins | jq -r ".[] | select(.focused) | $1"; }
  focus_app() { swaymsg "[app_id=\"$APP\"] focus" >/dev/null; }
else
  all_wins() { hyprctl clients -j; }
  active() { hyprctl activewindow -j | jq -r "$1"; }
  focus_app() { hyprctl dispatch "hl.dsp.focus({ window = \"class:$APP\" })" >/dev/null; }
fi

wins() { all_wins | jq "[.[] | select(.class==\"$APP\")]"; }
win_count() { wins | jq length; }

# The app is single-instance: launching while the user has Downer open would join
# their process, and killing it would close their windows.
require_no_instance() {
  [ -n "${DOWNER_HEADLESS:-}" ] && return
  if [ "$(win_count)" != 0 ]; then
    echo "Downer is already running; close it first (refusing to touch the user's windows)" >&2
    exit 2
  fi
  if pidof hyprlock >/dev/null; then echo "screen is locked; use scripts/manual/headless.sh" >&2; exit 2; fi
}

# use_workspace: on Hyprland, moves to the empty workspace ${WS:-7}, and back on exit.
use_workspace() {
  trap 'cleanup' EXIT
  [ -n "${DOWNER_HEADLESS:-}" ] && return
  PREV=$(hyprctl activeworkspace -j | jq -r .name)
  trap 'cleanup; hyprctl dispatch "hl.dsp.focus({ workspace = \"$PREV\" })" >/dev/null' EXIT
  hyprctl dispatch "hl.dsp.focus({ workspace = \"${WS:-7}\" })" >/dev/null; sleep 0.3
}

# launch [FILE…]: starts Downer on the files and focuses it, with XDG_CONFIG_HOME set
# to $CFG (default: an empty temp dir, so the user's config is never read). Waits for
# $WANT windows (default 1). Sets PID.
launch() {
  [ -x "$BIN" ] || { echo "run nix build first" >&2; exit 2; }
  XDG_CONFIG_HOME=${CFG:-$(mktemp -d)} "$BIN" "$@" >>"${LOG:-/dev/null}" 2>&1 &
  for _ in $(seq 25); do [ "$(win_count)" -ge "${WANT:-1}" ] && break; sleep 0.2; done
  sleep 0.6
  PID=$(wins | jq -r '.[0].pid')
  focus_app
  sleep 0.3
}

# keys WTYPE-ARGS...: sends keys only if a Downer window is focused.
keys() {
  if [ "$(active .class)" != "$APP" ]; then
    echo "Downer lost focus; stopping so keys don't go to another app" >&2
    cleanup; exit 3
  fi
  # Headless sway drops the first key after wtype loads its keymap; absorb it.
  if [ -n "${DOWNER_HEADLESS:-}" ]; then wtype -d 100 -k Shift_L "$@"; else wtype -d 100 "$@"; fi
  sleep 0.6
}

# float_size W H: floats the focused Downer window and sizes it in logical px.
float_size() {
  [ "$(active .class)" = "$APP" ] || { echo "Downer lost focus" >&2; cleanup; exit 3; }
  if [ -n "${DOWNER_HEADLESS:-}" ]; then
    swaymsg "[app_id=\"$APP\"] floating enable, resize set $1 $2, move position center" >/dev/null
  else
    hyprctl dispatch 'hl.dsp.window.float({ action = "enable" })' >/dev/null
    hyprctl dispatch "hl.dsp.window.resize({ x = $1, y = $2 })" >/dev/null
  fi
  sleep 0.6
}

shot() { grim -g "$(active '"\(.at[0]),\(.at[1]) \(.size[0])x\(.size[1])"')" "$1"; }

# differ A B [SKIP_TOP]: prints how many pixels differ between two screenshots, leaving
# out the overlay scrollbar (the right 40 px) and the top SKIP_TOP px (default 0).
differ() {
  local crop
  crop=$(magick identify -format "%[fx:w-40]x%[fx:h-${3:-0}]+0+${3:-0}" "$1")
  magick compare -metric AE <(magick "$1" -crop "$crop" png:-) <(magick "$2" -crop "$crop" png:-) null: 2>&1 |
    cut -d' ' -f1 | cut -d. -f1
}

cleanup() { for p in $(wins | jq -r '.[].pid' | sort -u); do kill "$p" 2>/dev/null; done; true; }

# kill_all: closes every test window (the instance is ours, see require_no_instance).
kill_all() { cleanup; for _ in $(seq 20); do [ "$(win_count)" = 0 ] && break; sleep 0.1; done; }

FAILS=0
check() { # check DESCRIPTION EXPECTED ACTUAL
  if [ "$2" = "$3" ]; then echo "PASS  $1"; else echo "FAIL  $1 (expected '$2', got '$3')"; FAILS=$((FAILS+1)); fi
}

# fixture DIR: writes sample.md, which uses every VIEW feature, and the images it shows.
fixture() {
  local d=$1
  magick -size 640x200 gradient:'#89b4fa-#cba6f7' "$d/local.png"
  cat >"$d/sample.md" <<'MD'
# Downer sample

A paragraph with *emphasis*, **strong**, `inline code`, ~~struck~~ text, a
[link to the end](#the-end), an [external link](https://example.org) and an
autolink <https://example.org/auto>. Escapes: \*not emphasis\* &amp; &copy;.
Hard break here  
and the next line.

a01 a02 a03 a04 a05 a06 a07 a08 a09 a10 a11 a12 a13 a14 a15 a16 a17 a18 a19 a20 a21 a22 a23 a24 a25 a26 a27 a28 a29 a30 a31 a32 a33 a34 a35 a36 a37 a38 a39 a40

## Second level

### Third level

#### Fourth level

##### Fifth level

###### Sixth level

> A quote.
> > A nested quote with **bold**.

- bullet one
- bullet two
  - nested bullet
    - deeper bullet

7. seven
8. eight
9. nine
10. ten

- [x] done task
- [ ] open task

```rust
fn main() {
    println!("a very long line of code that should scroll sideways and never wrap inside the code block at all, ever");
}
```

| Left | Centre | Right |
|:-----|:------:|------:|
| a | b | c |
| a longer cell with `code` | **bold** | 1234.56 |

---

<div class="raw">
  <b>raw html stays text</b>
</div>

Local image:

![a gradient](local.png)

Missing image: ![missing alt text](nope.png)

Remote image: ![remote alt text](https://example.org/x.png)

## The end

Last paragraph.
MD
}

# scroll DY: moves the pointer over the focused Downer window and scrolls by DY
# (wlrctl virtual pointer; positive is down).
scroll() {
  [ "$(active .class)" = "$APP" ] || { echo "Downer lost focus" >&2; cleanup; exit 3; }
  local x y
  x=$(active '.at[0] + .size[0] / 2 | floor'); y=$(active '.at[1] + .size[1] / 2 | floor')
  wlrctl pointer move -100000 -100000; wlrctl pointer move "$x" "$y"
  wlrctl pointer scroll "$1" 0; sleep 0.6
}

# click_at X Y: left-clicks at X,Y logical px from the focused Downer window's top left.
click_at() {
  [ "$(active .class)" = "$APP" ] || { echo "Downer lost focus" >&2; cleanup; exit 3; }
  local x y
  x=$(active ".at[0] + $1"); y=$(active ".at[1] + $2")
  wlrctl pointer move -100000 -100000; wlrctl pointer move "$x" "$y"
  wlrctl pointer click left; sleep 0.8
}

# hover_at X Y: moves the pointer to X,Y logical px from the focused window's top left.
hover_at() {
  local x y
  x=$(active ".at[0] + $1"); y=$(active ".at[1] + $2")
  wlrctl pointer move -100000 -100000; wlrctl pointer move "$x" "$y"
}
