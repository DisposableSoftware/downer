#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq grim sway desktop-file-utils glib
# Spec: INT-2, INT-3. Run headless (./scripts/manual/headless.sh ./scripts/manual/xdg-open.sh)
# after `nix build`. Uses a temp mimeapps.list, so the user's defaults are untouched.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
[ -n "${DOWNER_HEADLESS:-}" ] || { echo "run under headless.sh" >&2; exit 2; }
D=$APP.desktop
check "desktop entry installed" 1 "$([ -f result/share/applications/$D ] && echo 1)"
check "example config installed" 1 "$([ -f result/share/doc/downer/config.example.ini ] && echo 1)"
check "desktop entry validates" 0 "$(desktop-file-validate result/share/applications/$D | grep -vc hint)"
T=$(mktemp -d); export XDG_CONFIG_HOME=$T/config XDG_DATA_HOME=$T/data
# GIO maps MIME types to apps through mimeinfo.cache, which NixOS and home-manager
# profiles generate; do the same for a temp data dir.
mkdir -p "$XDG_DATA_HOME/applications"; cp "result/share/applications/$D" "$XDG_DATA_HOME/applications/"
update-desktop-database "$XDG_DATA_HOME/applications"
export PATH=$PWD/result/bin:$PATH
check "offered for text/markdown" 1 "$(gio mime text/markdown | grep -q "^	$D$" && echo 1)"
gio mime text/markdown "$D" >/dev/null
printf '# Opened\n' >"$T/opened.md"
use_workspace
gio open "$T/opened.md"
for _ in $(seq 25); do [ "$(win_count)" -gt 0 ] && break; sleep 0.2; done
check "gio open on a .md opens Downer" "opened.md — Downer" "$(wins | jq -r '.[0].title')"
exit $FAILS
