#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq wtype grim imagemagick sway
# Makes the README screenshots in docs/screenshots from a demo folder, after `nix build`:
#   DOWNER_RES=3200x2000 scripts/manual/headless.sh scripts/screenshots.sh
# Windows are 1600×1000 logical px at scale 2, downscaled to 1600×1000. Everything
# shown is made up here, in a temp dir.
cd "$(dirname "$0")/.." || exit 1
. scripts/manual/lib.sh
[ -n "${DOWNER_HEADLESS:-}" ] || { echo "run under scripts/manual/headless.sh" >&2; exit 2; }
OUT=docs/screenshots; mkdir -p "$OUT"; T=$(mktemp -d); D=$T/field-guide
mkdir -p "$D/getting-started" "$D/recipes" "$D/notes"

cat >"$D/README.md" <<'MD'
# Field guide

A small handbook for the **Downer** demo. Everything here is made up, and none of it is
advice. Start with [installing](getting-started/install.md), or jump to
[the recipes](recipes/sourdough.md#method).

> Markdown is just text with opinions. Downer has opinions about colours.

## What's inside

| Section | Pages | Status |
|:--------|:-----:|-------:|
| Getting started | 2 | done |
| Recipes | 2 | simmering |
| Notes | 3 | ~~tidy~~ messy |

## Today

- [x] Render Markdown in Catppuccin colours
- [x] Follow the folder as files come and go
- [ ] Learn to make bread

```toml
[appearance]
flavor = "mocha"
accent = "mauve"
```
MD
cat >"$D/getting-started/install.md" <<'MD'
# Installing

Run it once without installing anything:

```sh
nix run github:DisposableSoftware/downer
```

Then open a file or a whole folder:

```sh
downer README.md
downer ~/notes
```
MD
printf '# Configuring\n\nColours live in `~/.config/downer/config.ini`.\n' >"$D/getting-started/configure.md"
cat >"$D/recipes/sourdough.md" <<'MD'
# Sourdough, roughly

1. Feed the starter the night before.
2. Mix **500 g** flour, **350 g** water and **100 g** starter.
3. Wait. Then wait some more.

## Method

Fold the dough every half hour for two hours, shape it, and leave it in the fridge
overnight. Bake at 250 °C in a covered pot: 20 minutes with the lid, 25 without.

> If it doesn't rise, call it a flatbread.
MD
printf '# Soup\n\nSee [sourdough](sourdough.md) for something to dip.\n' >"$D/recipes/soup.md"
for n in 1 2 10; do printf '# Day %s\n\nNothing happened.\n' "$n" >"$D/notes/day-$n.md"; done
printf '# Changelog\n\n## 0.1.0\n\n- First release.\n' >"$D/CHANGELOG.md"

# shoot NAME FLAVOR ACCENT TARGET [KEYS…]: a window on TARGET, after KEYS, into OUT/NAME.png.
shoot() {
  local name=$1 flavor=$2 accent=$3 target=$4; shift 4
  CFG=$T/cfg-$name; mkdir -p "$CFG/downer"
  printf '[appearance]\nflavor=%s\naccent=%s\nopacity=1\n' "$flavor" "$accent" >"$CFG/downer/config.ini"
  CFG=$CFG launch "$target"
  for k in "$@"; do keys -k "$k"; done
  sleep 0.5; shot "$T/$name.png"; kill_all
  magick "$T/$name.png" -resize 1600x1000 -strip "$OUT/$name.png"
  echo "$OUT/$name.png"
}

use_workspace
# Up ×4 from README reaches getting-started; Return opens it.
shoot folder mocha mauve "$D" Up Up Up Up Return
shoot document mocha mauve "$D/README.md"
shoot latte latte blue "$D" Up Up Return Down Down Return
shoot frappe frappe peach "$D/getting-started/install.md"
