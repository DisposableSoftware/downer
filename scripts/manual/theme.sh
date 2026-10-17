#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim sway
# Spec: LOOK-4, LOOK-5, LOOK-8, CONF-5. Run headless after `nix build`
# (./scripts/manual/headless.sh ./scripts/manual/theme.sh). Screenshots go to $OUT.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
require_no_instance
OUT=${OUT:-$(mktemp -d)}; T=$(mktemp -d); LOG=$OUT/stderr.log
printf '# Heading\n\nBody with a [link](#heading) and `code`.\n\n```\ncode block\n```\n' >"$T/t.md"
use_workspace

# LOOK-4: user CSS can use palette names, and wins over the app's rules.
CFG=$T/user; mkdir -p "$CFG/downer"
printf 'window.downer .h1 { color: @red; }\n' >"$CFG/downer/style.css"
CFG=$CFG launch "$T/t.md"; shot "$OUT/user-css.png"; kill_all

# LOOK-5: flavor = gtk with GTK's built-in theme, which only has the theme_* names.
CFG=$T/gtk; mkdir -p "$CFG/downer"; printf '[appearance]\nflavor=gtk\n' >"$CFG/downer/config.ini"
GTK_THEME=Default CFG=$CFG launch "$T/t.md"; shot "$OUT/gtk-default.png"
# LOOK-8: the selection keeps `text` colour whatever the theme's selected text is.
keys -k Tab; shot "$OUT/gtk-default-selected.png"; kill_all
check "no GTK or CSS warnings" 0 "$(grep -iE 'warning|critical' "$LOG" | grep -vc VK_SUBOPTIMAL)"
echo "screenshots in $OUT"
exit $FAILS
