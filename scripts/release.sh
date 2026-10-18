#!/usr/bin/env bash
# Bumps the version, commits "Downer X.Y.Z" and tags vX.Y.Z. Does not push.
# Writes draft release notes to NOTES_FILE for `gh release create -F`.
# When Cargo.toml already has the version (the first release), it tags HEAD without a
# bump commit.
# Usage: scripts/release.sh patch|minor|major|X.Y.Z NOTES_FILE
set -euo pipefail
cd "$(dirname "$0")/.."

[ $# = 2 ] || { echo "usage: $0 patch|minor|major|X.Y.Z NOTES_FILE" >&2; exit 2; }
git diff --quiet && git diff --cached --quiet || { echo "working tree is dirty" >&2; exit 1; }

old=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
IFS=. read -r major minor patch <<<"$old"
case $1 in
  major) new="$((major + 1)).0.0" ;;
  minor) new="$major.$((minor + 1)).0" ;;
  patch) new="$major.$minor.$((patch + 1))" ;;
  [0-9]*.[0-9]*.[0-9]*) new=$1 ;;
  *) echo "bad level: $1" >&2; exit 2 ;;
esac
git rev-parse -q --verify "refs/tags/v$new" >/dev/null && { echo "v$new already exists" >&2; exit 1; }
prev=$(git describe --tags --abbrev=0 --match 'v[0-9]*' 2>/dev/null || true)
ids=$(scripts/release-check.sh | tail -n +2 | grep -E '^[A-Z]+-[0-9]+ is now ' | cut -d' ' -f1 || true)

if [ "$new" != "$old" ]; then
  sed -i "0,/^version = \"$old\"/s//version = \"$new\"/" Cargo.toml
  sed -i "/^name = \"downer\"$/{n;s/^version = \"$old\"/version = \"$new\"/}" Cargo.lock
  nix develop -c cargo test --quiet >/dev/null 2>&1 || { git checkout -- .; echo "cargo test failed; version left at $old" >&2; exit 1; }
  git commit -q -am "Downer $new"
fi
git tag -a "v$new" -m "Downer $new"

changes=$(
  if [ -n "$prev" ]; then
    for id in $ids; do
      row=$(grep -h "^| \`$id\` | " spec/[!R]*.md | head -1 | awk -F' [|] ' '{print $2}')
      echo "- \`$id\`: $row"
    done
    git log --format='- %s' "$prev..v$new" | grep -v -e '^- spec: ' -e '^- Downer '
  else
    echo "First release: a Markdown viewer that renders files and folders in Catppuccin colours."
  fi
)
cat >"$2" <<EOF
> [!WARNING]
> **Certified, artisanal, vibe-coded slop.** An AI wrote all of it while a person said "yeah, looks good" and pressed Enter a lot. It only reads your Markdown, so the worst it can do is render it badly.

## Changes

$changes

## Install

\`\`\`sh
nix run github:DisposableSoftware/downer/v$new
\`\`\`

See the [README](https://github.com/DisposableSoftware/downer/blob/v$new/README.md).
EOF
echo "v$new"
