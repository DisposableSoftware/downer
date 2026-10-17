<h1 align="center">Downer</h1>

<p align="center">A small GTK 4 Markdown viewer for Hyprland.</p>

> [!WARNING]
> **Certified, artisanal, vibe-coded slop.**
>
> No human has read this code. An AI writes all of it while a person says "yeah, looks
> good" and presses Enter a lot. It shows Markdown files in Catppuccin colours and
> re-renders them when they change, which is more than anyone expected. If you need to
> read Markdown seriously, any text editor will do a better job.
>
> Bug reports will be read with great affection and then ignored.

## Building

```sh
nix run github:DisposableSoftware/downer
```

or, from a checkout:

```sh
nix develop
cargo run
```

## Using it

```sh
downer README.md notes.md   # one window per file
```

Make it the default for Markdown with `xdg-mime default io.github.DisposableSoftware.Downer.desktop text/markdown`,
and `xdg-open` will use it. Colours are set in `~/.config/downer/config.ini` (created on first run,
everything commented out): `flavor` is latte, frappe, macchiato, mocha or `gtk` to follow your GTK
theme, plus `accent` and `opacity`. Extra CSS goes in `~/.config/downer/style.css`.

## Development

What Downer does is written down in [`spec/`](spec/README.md): every behaviour is a
numbered requirement, and the code follows it. `cargo test` checks the spec's tables
against the code, and `nix flake check` also runs clippy and rustfmt.

## License

MIT, see [LICENSE](LICENSE).
