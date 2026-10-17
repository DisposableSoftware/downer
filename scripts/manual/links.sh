#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick wlrctl
# Spec: VIEW-6 (fragment links). Run on Hyprland (headless sway has no pointer to click
# with) from the repo root after `nix build`, with no Downer windows open.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; T=$(mktemp -d)
{ echo "[jump](#middle) [nowhere](#no-such-heading)"; for i in $(seq 60); do echo; echo "Paragraph $i."; done
  echo; echo "## Middle"; for i in $(seq 61 120); do echo; echo "Paragraph $i."; done; } >"$T/d.md"
use_workspace
launch "$T/d.md"
float_size 900 700; sleep 1 # let the float animation finish
shot "$OUT/before.png"
hover_at 60 34; sleep 1.5
grim -g "$(active '"\(.at[0]),\(.at[1]) \(.size[0])x200"')" "$OUT/tooltip.png"
hover_at 600 400; sleep 0.5
# The column is centred: 900 - 2 × 24 px padding is wider than 800, so it starts at 50.
click_at 120 34; shot "$OUT/nowhere.png"
# The click puts a text cursor in the link row, so compare below it (at scale 2).
check "unknown fragment does nothing" 0 "$(differ "$OUT/before.png" "$OUT/nowhere.png" 120)"
click_at 60 34; shot "$OUT/jump.png"
check "fragment link scrolls" 1 "$([ "$(differ "$OUT/before.png" "$OUT/jump.png")" -gt 0 ] && echo 1)"
echo "screenshots in $OUT (jump.png should start at the Middle heading)"
exit $FAILS
