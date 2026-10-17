#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick sway wlrctl
# Spec: NAV-5. Run from the repo root after `nix build`, with no Downer windows open.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; T=$(mktemp -d); F=$T/doc.md
{ echo "# Reload"; for i in $(seq 300); do echo; echo "Paragraph $i."; done; } >"$F"
use_workspace
launch "$F"
scroll 30
shot "$OUT/a.png"

echo $'\nAppended at the end.' >>"$F"; sleep 1
shot "$OUT/b.png"
check "append below the view keeps the scroll position" 0 "$(differ "$OUT/a.png" "$OUT/b.png")"

{ echo "# Reload"; for i in $(seq 300); do echo; echo "Paragraph $i (renamed in)."; done; } >"$T/tmp"
mv "$T/tmp" "$F"; sleep 1
shot "$OUT/c.png"
check "atomic rename re-renders" 1 "$([ "$(differ "$OUT/b.png" "$OUT/c.png")" -gt 0 ] && echo 1)"

{ echo "# Short"; echo; echo "Only this."; } >"$T/tmp"; mv "$T/tmp" "$F"; sleep 1
shot "$OUT/d.png"
check "shorter file clamps the scroll" 1 "$([ "$(differ "$OUT/c.png" "$OUT/d.png")" -gt 0 ] && echo 1)"

rm "$F"; sleep 1
shot "$OUT/e.png"
check "delete keeps the last version" 0 "$(differ "$OUT/d.png" "$OUT/e.png")"

{ echo "# Back"; echo; echo "Recreated."; } >"$F"; sleep 1
shot "$OUT/f.png"
check "recreated file is rendered" 1 "$([ "$(differ "$OUT/e.png" "$OUT/f.png")" -gt 0 ] && echo 1)"
echo "screenshots in $OUT"
exit $FAILS
