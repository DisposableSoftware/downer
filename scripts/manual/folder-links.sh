#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick wlrctl
# Spec: NAV-11. Run on Hyprland (headless sway has no pointer to click with) from the
# repo root after `nix build`, with no Downer windows open. Screenshots go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; mkdir -p "$OUT"; T=$(mktemp -d); N=$T/notes
mkdir -p "$N/sub"
# The link text is long so that it is under the click point whatever the sidebar's
# width: in a 1400px window the column starts at 300 + sidebar/2 (320 to 440).
printf '[%s](sub/other.md#part)\n' "$(printf 'go to the part %.0s' 1 2 3 4 5 6)" >"$N/README.md"
{ echo "# Other"; for i in $(seq 80); do echo; echo "Line $i."; done; echo; echo "## Part"; echo; echo "Here."; } >"$N/sub/other.md"
use_workspace
launch "$N"
float_size 1400 800; sleep 1
check "starts on the README" "README.md — notes — Downer" "$(wins | jq -r '.[0].title')"
shot "$OUT/before.png"
click_at 560 34
check "the link opens the file in the window" "other.md — notes — Downer" "$(wins | jq -r '.[0].title')"
check "...without opening another window" 1 "$(win_count)"
shot "$OUT/after.png"
echo "screenshots in $OUT (after.png: the sub/other.md row is current, and the view starts at Part)"
exit $FAILS
