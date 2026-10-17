#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim wl-clipboard sway
# Spec: VIEW-9. Run headless (./scripts/manual/headless.sh ./scripts/manual/select.sh),
# which has its own clipboard; on Hyprland this would overwrite the user's clipboard.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
[ -n "${DOWNER_HEADLESS:-}" ] || { echo "run under headless.sh (it uses the clipboard)" >&2; exit 2; }
T=$(mktemp -d)
printf 'A *styled* paragraph.\n\n```\nfn main() {\n    x();\n}\n```\n' >"$T/s.md"
use_workspace
launch "$T/s.md"
# Tab focuses the next selectable block, which selects all its text.
keys -k Tab; keys -M ctrl -k c -m ctrl
check "paragraph copied as plain text" "A styled paragraph." "$(wl-paste -n)"
keys -k Tab; keys -M ctrl -k c -m ctrl
check "code block copied with its whitespace" $'fn main() {\n    x();\n}' "$(wl-paste -n)"
exit $FAILS
