#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick sway
# Spec: NAV-2, NAV-4, NAV-6…NAV-10, LOOK-9, LOOK-10. Run headless after `nix build`:
# ./scripts/manual/headless.sh ./scripts/manual/folder.sh. Screenshots go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; mkdir -p "$OUT"; T=$(mktemp -d); LOG=$OUT/stderr.log
title() { wins | jq -r '.[0].title'; }

# sidebar_edge PNG Y: the sidebar's width in logical px, found on row Y (logical) as the
# first column, from the left, whose colour is the main view's (scale 2).
sidebar_edge() {
  magick "$1" -crop "x1+0+$(($2 * 2))" +repage -depth 8 txt:- | awk -F'[,: ]+' '
    NR == 1 { next } { col[$1] = $NF } END {
      bg = col[1800]; for (x = 0; x < 1800; x++) if (col[x] == bg) { print int(x / 2); exit } }'
}
# text_right PNG Y0 Y1: the rightmost text pixel (logical) left of 300px between rows.
text_right() {
  magick "$1" -crop "600x$((($3 - $2) * 2))+0+$(($2 * 2))" +repage -fuzz 25% -fill black \
    -opaque "$(magick "$1" -format '%[pixel:p{1,1}]' info:)" -trim -format '%[fx:int((page.x + w) / 2)]' info:
}

N=$T/notes; mkdir -p "$N/sub/deeper" "$N/.hidden" "$N/empty" "$N/a-folder-with-a-really-quite-long-name-for-the-sidebar"
printf '# Notes\n\nThe readme.\n' >"$N/README.md"
printf '# B2\n' >"$N/B2.md"; printf '# b10\n' >"$N/b10.md"; echo x >"$N/notes.txt"
echo x >"$N/.hidden/x.md"; echo x >"$N/empty/a.png"
printf '# Other\n' >"$N/sub/other.md"; printf '# Deep\n\nDeep text.\n' >"$N/sub/deeper/deep.md"
printf '# Long\n' >"$N/a-folder-with-a-really-quite-long-name-for-the-sidebar/x.md"
use_workspace

# NAV-6, NAV-7, NAV-9: README opens with its row current; LOOK-10: the long name caps the width.
launch "$N"
check "README opens; folder title" "README.md — notes — Downer" "$(title)"
shot "$OUT/notes.png"
check "long name caps the sidebar at 280px" 280 "$(sidebar_edge "$OUT/notes.png" 900)"

# NAV-8: focus starts on the current row; Up, Down and Enter drive the tree.
for k in Up Up Up Up Return Down Down Return Down Return Down Return; do keys -k "$k"; done
check "keyboard opens a file two folders down" "deep.md — notes — Downer" "$(title)"
shot "$OUT/keys.png"

# NAV-10: the tree follows the folder; expanded folders and the current row stay.
printf '# New\n' >"$N/sub/new.md"; sleep 1; shot "$OUT/live-new.png"
mkdir "$N/added"; printf '# A\n' >"$N/added/a.md"; sleep 1; shot "$OUT/live-folder.png"
mv "$N/b10.md" "$N/sub/moved.md"; sleep 1; shot "$OUT/live-moved.png"
check "changes keep the shown file" "deep.md — notes — Downer" "$(title)"
cp "$OUT/live-moved.png" "$OUT/before-delete.png"
rm "$N/sub/deeper/deep.md"; sleep 1; shot "$OUT/live-deleted.png"
magick "$OUT/before-delete.png" -crop 2000x800+1000+0 +repage "$OUT/main-before.png"
magick "$OUT/live-deleted.png" -crop 2000x800+1000+0 +repage "$OUT/main-after.png"
check "deleting the shown file keeps the rendering" 0 "$(differ "$OUT/main-before.png" "$OUT/main-after.png")"
check "...and changes the tree" 1 \
  "$([ "$(differ "$OUT/before-delete.png" "$OUT/live-deleted.png")" -gt 0 ] && echo 1)"
kill_all

# NAV-9 hints, NAV-6 errors, NAV-2 file windows.
P=$T/plain; mkdir -p "$P/f"; printf '# A\n' >"$P/a.md"; printf '# B\n' >"$P/b.md"
printf '# L\n' >"$P/f/a-longer-name.md"
launch "$P"; check "no README: folder title" "plain — Downer" "$(title)"; shot "$OUT/plain.png"
# LOOK-10: 16px from the widest row's text to the edge (give or take the antialiased
# column past the last glyph), and expanding widens the sidebar.
near16() { [ "$1" -ge 15 ] && [ "$1" -le 17 ] && echo 1; }
w0=$(sidebar_edge "$OUT/plain.png" 900); r0=$(text_right "$OUT/plain.png" 8 120)
check "16px right of the widest name ($((w0 - r0)))" 1 "$(near16 $((w0 - r0)))"
keys -k Up; keys -k Up; keys -k Return; shot "$OUT/plain-expanded.png"
w1=$(sidebar_edge "$OUT/plain-expanded.png" 900); r1=$(text_right "$OUT/plain-expanded.png" 8 160)
check "expanding a longer name widens it" 1 "$([ "$w1" -gt "$w0" ] && echo 1)"
check "...still 16px right of it ($((w1 - r1)))" 1 "$(near16 $((w1 - r1)))"
kill_all
mkdir "$T/none"; echo x >"$T/none/a.txt"
launch "$T/none"; check "no Markdown: folder title" "none — Downer" "$(title)"; shot "$OUT/none.png"; kill_all
mkdir "$T/locked"; chmod 000 "$T/locked"
launch "$T/locked"; check "unreadable folder: still a window" 1 "$(win_count)"; shot "$OUT/locked.png"; kill_all
chmod 700 "$T/locked"
launch "$P/a.md"; check "file window title" "a.md — Downer" "$(title)"; shot "$OUT/file.png"; kill_all
check "no GTK warnings" 0 "$(grep -iE 'warning|critical' "$LOG" | grep -vc VK_SUBOPTIMAL)"
echo "screenshots in $OUT"
exit $FAILS
