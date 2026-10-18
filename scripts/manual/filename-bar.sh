#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick sway
# Spec: VIEW-11, LOOK-13. Run headless after `nix build`:
# ./scripts/manual/headless.sh ./scripts/manual/filename-bar.sh. Screenshots go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; mkdir -p "$OUT"; T=$(mktemp -d); LOG=$OUT/stderr.log

# ink PNG X0: 1 if the strip holds text, else 0. The background is the pixel at (X0+590, 1).
ink() {
  local bg; bg=$(magick "$1" -format "%[pixel:p{$((2 * $2 + 1180)),1}]" info:)
  magick "$1" -crop "1200x60+$(($2 * 2))+0" +repage -fuzz 15% -fill black -opaque "$bg" \
    -fill white +opaque black -format '%[fx:mean > 0 ? 1 : 0]' info:
}
# left PNG X0: the leftmost text pixel (logical) in the strip, relative to X0.
left() {
  local bg; bg=$(magick "$1" -format "%[pixel:p{$((2 * $2 + 1180)),1}]" info:)
  magick "$1" -crop "1200x60+$(($2 * 2))+0" +repage -fuzz 15% -fill black -opaque "$bg" \
    -fill white +opaque black -trim -format '%[fx:int(page.x / 2)]' info:
}

N=$T/notes; mkdir -p "$N/sub" "$T/empty"
printf '# Notes\n\nThe readme.\n' >"$N/README.md"
printf '# Other\n\nOther text.\n' >"$N/sub/other.md"
L="a-file-name-that-is-far-too-long-to-fit-in-a-narrow-window-so-it-is-cut-at-the-start-end-of-name.md"
printf '# Long\n' >"$T/$L"
use_workspace

# VIEW-11, LOOK-13: a file window shows the name; text starts 16px from the edge.
launch "$N/README.md"
shot "$OUT/file.png"
check "file window: bar has text" 1 "$(ink "$OUT/file.png" 0)"
l=$(left "$OUT/file.png" 0)
check "text starts 16px from the left ($l)" 1 "$([ "$l" -ge 15 ] && [ "$l" -le 19 ] && echo 1)"
# The bar doesn't take focus: Tab goes to the first selectable block, never the bar.
keys -k Tab; shot "$OUT/tab.png"
kill_all

# VIEW-11: a long name is ellipsized at the start, so the end of the name stays.
launch "$T/$L"; float_size 500 400; shot "$OUT/long.png"
check "long name: window stays 500px" 500 "$(active '.size[0]')"
kill_all

# VIEW-11: folder window, bar over the main view only; changes with the file.
launch "$N"
shot "$OUT/folder-readme.png"
check "folder window: title" "README.md — notes — Downer" "$(wins | jq -r '.[0].title')"
check "folder window: bar has text right of the sidebar" 1 "$(ink "$OUT/folder-readme.png" 150)"
keys -k Up; keys -k Return; keys -k Down; keys -k Return; shot "$OUT/folder-other.png"
check "another file: the bar changes" 1 "$([ "$(differ "$OUT/folder-readme.png" "$OUT/folder-other.png" 0)" -gt 0 ] && echo 1)"
kill_all

# VIEW-11: no bar on the hint page, with no file shown, or for an unreadable file.
launch; shot "$OUT/hint.png"; check "hint page: no bar" 0 "$(ink "$OUT/hint.png" 0)"; kill_all
launch "$T/empty"; shot "$OUT/empty.png"; check "folder with no file shown: no bar" 0 "$(ink "$OUT/empty.png" 150)"; kill_all
launch "$T/missing.md"; shot "$OUT/missing.png"; kill_all

check "no GTK warnings" 0 "$(grep -iE 'warning|critical' "$LOG" | grep -vc VK_SUBOPTIMAL)"
echo "screenshots in $OUT"
exit $FAILS
