#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim
# Spec: PROD-3. Run from the repo root after `nix build`, with no Downer windows open.
# Opens Downer on workspace ${WS:-7} and checks the window, then goes back.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance

use_workspace

launch
check "a window opens"   1 "$(win_count)"
check "titled Downer"    Downer "$(wins | jq -r '.[0].title')"
exit $FAILS
