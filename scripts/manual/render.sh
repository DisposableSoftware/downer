#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick sway
# Spec: VIEW-1…VIEW-8, LOOK-1, LOOK-2, LOOK-5…LOOK-7. Run from the repo root after
# `nix build`, with no Downer windows open. Screenshots of sample.md per flavor, at full
# size and 500px wide, go to $OUT (default: a temp dir). Headless, use a tall output so the
# whole sample fits: DOWNER_RES=3840x5000 ./scripts/manual/headless.sh ./scripts/manual/render.sh
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; T=$(mktemp -d); LOG=$OUT/stderr.log
fixture "$T"
use_workspace
for flavor in mocha latte frappe macchiato gtk; do
  CFG=$T/cfg-$flavor; mkdir -p "$CFG/downer"
  printf '[appearance]\nflavor=%s\naccent=%s\n' "$flavor" "${ACCENT:-mauve}" >"$CFG/downer/config.ini"
  CFG=$CFG launch "$T/sample.md"
  check "$flavor: titled" "sample.md — Downer" "$(wins | jq -r '.[0].title')"
  shot "$OUT/$flavor.png"
  if [ "$flavor" = mocha ]; then
    float_size 500 2400; shot "$OUT/$flavor-narrow.png"
    check "narrow: page doesn't scroll sideways" 500 "$(active '.size[0]')"
  fi
  kill_all
done
check "no GTK or Pango warnings" 0 "$(grep -iE 'warning|critical' "$LOG" | grep -vc VK_SUBOPTIMAL)"
echo "screenshots in $OUT"
exit $FAILS
