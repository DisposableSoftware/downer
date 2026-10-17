#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick sway
# Spec: NAV-1…NAV-4, PROD-3. Run from the repo root after `nix build`, with no Downer
# windows open. Screenshots go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; T=$(mktemp -d)
printf '# One\n' >"$T/one.md"; printf '# Two\n' >"$T/two.md"; printf '# Three\n' >"$T/three.md"
use_workspace

WANT=2 launch "$T/one.md" "$T/two.md"
check "two files, two windows" 2 "$(win_count)"
check "titles" "one.md — Downer,two.md — Downer" "$(wins | jq -r '[.[].title] | sort | join(",")')"
(cd "$T" && XDG_CONFIG_HOME=$T "$OLDPWD/$BIN" three.md) # relative path, while running
sleep 1
check "second launch adds a window" 3 "$(win_count)"
check "...in the same process" 1 "$(wins | jq '[.[].pid] | unique | length')"
check "relative path resolved" 1 "$(wins | jq '[.[] | select(.title == "three.md — Downer")] | length')"
kill_all

launch
check "no arguments: titled Downer" Downer "$(wins | jq -r '.[0].title')"
shot "$OUT/empty.png"
kill_all

launch "$T/missing.md"
check "missing file: titled by name" "missing.md — Downer" "$(wins | jq -r '.[0].title')"
shot "$OUT/missing.png"
kill_all

mkdir "$T/folder.md"
launch "$T/folder.md"
shot "$OUT/folder.png"
check "folder: still running" 1 "$(win_count)"
kill_all
echo "screenshots in $OUT"
exit $FAILS
