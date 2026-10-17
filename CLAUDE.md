# Downer

A minimal GTK 4 Markdown viewer for Linux, built for Hyprland. Display name **Downer**, binary `downer`, app ID `io.github.DisposableSoftware.Downer`. Published under the DisposableSoftware GitHub account as tongue-in-cheek "vibe-coded slop": keep that framing in the README and anything public.

**What the app does is defined in [`spec/`](spec/README.md).** This file covers how to work on it.

## Spec-first workflow (mandatory)

Every change follows these steps, in order. For feature requests, use the `/feature` skill (`.claude/skills/feature/`), which adds a user-approval step between the spec and the code. For bugs, use `/bug` (`.claude/skills/bug/`): it reproduces the bug, finds the spec gap that let it happen, then commits the spec delta and the fix without an approval step, unless the fix changes intended behaviour.

1. **Spec.** Before touching code, update the relevant `spec/*.md`.
   - **New behaviour or a changed requirement:** add or edit a requirement row. A new row gets the next free ID in its prefix, with status `planned`. If the request is unclear, settle the wording with the user before coding.
   - **Bug fix:** find the requirement the bug breaks. If no requirement covers the behaviour, the bug is a gap in the spec, so add one.
   - **Refactor with no behaviour change:** no spec edit, but it must still pass step 3.
   - Commit the spec change on its own, before the code: `spec: <summary>`.
2. **Code.** Implement only what the spec says. If the code needs behaviour the spec doesn't describe, go back to step 1.
3. **Confirm against the spec.** Before the code commit:
   - Run `cargo test`. It includes `src/spec_tests.rs`, which checks the spec's requirement rows and any machine-checked tables against the code.
   - For every requirement ID touched, verify it as its **Verified by** column says (unit test, or a manual/screenshot check on Hyprland). Prefer adding a unit test where one is possible.
   - Re-read the changed code against the requirement text and look for behaviour the spec doesn't mention.
   - Update the row: set the status to `done` (or `partial` with the gap written in), and fill in **Verified by**. Update `roadmap.md` if a planned item landed.
4. **Commit** the code with a `Spec: ID, ID` trailer listing the requirements it implements or affects, and tell the user which IDs were verified and how.

When reporting to the user, say plainly if something could only be partly verified (e.g. the user had Downer open, so the manual test was skipped).

## Commands

```bash
nix develop                    # devshell: cargo, clippy, rustfmt, rust-analyzer, gtk4
cargo run                      # run (inside devshell)
cargo clippy --all-targets -- -D warnings    # must pass
cargo fmt
cargo test                     # unit tests + spec conformance tests
nix build                      # package -> ./result/bin/downer (also runs the tests)
nix flake check                # tests, clippy and rustfmt
```

The flake only sees git-tracked files. Run `git add` on new files before `nix build` or `nix develop`. After adding crates, commit `Cargo.lock`, which `nix/package.nix` reads through `cargoLock.lockFile`. `spec/` is part of the package source, because the spec tests read it.

## Code layout

```
src/main.rs        GtkApplication (HANDLES_OPEN), prgname                    PROD-3, NAV-1
src/view.rs        windows, loading, reload monitor, block widgets, links   VIEW NAV
src/render.rs      Markdown -> block tree (no GTK), anchors, link resolving VIEW
src/column.rs      max-width centred column widget                          VIEW-2
src/theme.rs       Catppuccin palette or GTK theme colours, CSS              LOOK
src/config.rs      config.ini, first-run file                               CONF
src/spec_tests.rs  spec conformance tests (row format, config table)
data/              style.css, themes/<flavor>.css, config.example.ini, desktop entry
scripts/manual/    UI checks (lib.sh helpers; headless.sh runs one in a headless sway)
nix/               package.nix
spec/              the specification
```

## Conventions

- Keep the dependency list short. Ask before adding a crate, and prefer GIO/GLib equivalents (`PROD-1`).
- Use async GIO APIs for file I/O. Never block the main loop.
- Avoid strong reference cycles in signal closures: use `glib::clone!` with `#[weak]` for widgets and models.
- Look up gtk4-rs and GTK docs before guessing at APIs, because the bindings change between minor versions. Read the vendored source in `~/.cargo/registry/src/*/gtk4-0.11.*` for exact signatures.
  - https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/
  - https://docs.gtk.org/gtk4/
- Comments only when the reason is not obvious.
- Unit tests that use GTK must call `gtk::init()` and skip if it fails, because the `nix build` sandbox has no display. GTK may only be initialised on one thread, so keep GTK checks in one test.
- Before anything goes public: no personal information (names, emails, home paths, hostnames) in code, fixtures, screenshots or commits. Commits are authored as DisposableSoftware's GitHub noreply address (set in the repo's git config).

## Manual testing

Prefer adding a script to `scripts/manual/` (source `lib.sh`, which has the guards below built in) over ad-hoc commands. Each script names the spec IDs it checks. Run it after `nix build`.

### Headless (default)

Run checks with `./scripts/manual/headless.sh ./scripts/manual/<name>.sh`. It starts a headless sway with its own D-Bus session, runtime dir and clipboard, so it works while the screen is locked or the user has Downer open, and never touches their windows. `lib.sh` switches to `swaymsg` when `DOWNER_HEADLESS` is set. The output is 3840×2160 at scale 2; set `DOWNER_RES` for a taller one (`render.sh` uses `3840x5000` to fit the whole sample).

- The headless seat has no lasting pointer: each `wlrctl` call adds a virtual pointer and removes it, so GTK never sees its scrolls or clicks. Checks that scroll or click (`reload.sh`, `links.sh`) run on Hyprland.
- Enter doesn't activate a link in a selectable label, and Tab into a selectable label selects all its text (`select.sh` relies on that).
- `magick compare -metric AE` prints a float; `differ` in `lib.sh` cuts it to an integer and leaves out the overlay scrollbar.

### On Hyprland

Run `./scripts/manual/<name>.sh` directly.

- Run visual tests on an empty workspace and switch back afterwards, and ask before a test that takes focus if the user is at the machine.
- Screenshots: `hyprctl clients -j` gives the window geometry, and `grim -g` captures it. Use an `XDG_CONFIG_HOME` in a temp dir to test config variants, and sample files in a temp dir, never the user's own.
- Check `pidof hyprlock` first: if the screen is locked, grim captures the lock screen.
- New windows don't take focus. Run `hyprctl dispatch 'hl.dsp.focus({ window = "class:io.github.DisposableSoftware.Downer" })'` (Hyprland 0.56 uses a Lua config), then send keys with `wtype`. Always check `hyprctl activewindow` first, so keys never land in the user's other apps.
- The Nix wrapper renames the process to `.downer-wrapped`, so `pgrep -x downer` never matches, and `pgrep -f` matches the test's own shell. Find and kill test windows through `hyprctl clients -j` (`.pid`).
- The app is single-instance. If the user already has Downer open, a test launch only opens a window in *their* process, and killing that pid closes their windows. Check there are no `io.github.DisposableSoftware.Downer` clients before testing, and don't go ahead if there are.
