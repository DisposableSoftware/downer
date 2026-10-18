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

## Installing on NixOS

Add the flake as an input:

```nix
inputs.downer.url = "github:DisposableSoftware/downer";
```

System-wide, with the NixOS module:

```nix
imports = [ inputs.downer.nixosModules.default ];
programs.downer = {
  enable = true;
  defaultMarkdownViewer = true; # xdg-open file.md opens Downer
};
```

Or per user, with the home-manager module, which can also write the config:

```nix
imports = [ inputs.downer.homeManagerModules.default ];
programs.downer = {
  enable = true;
  settings.appearance = {
    flavor = "latte";
    accent = "blue";
    opacity = 0.9;
  };
  style = "window.downer .h1 { color: @red; }";
  defaultMarkdownViewer = true;
};
xdg.mimeApps.enable = true; # home-manager only writes mimeapps.list when this is set
```

If you've run Downer before, it has already made a commented-out `~/.config/downer/config.ini`,
and home-manager will refuse to replace it. Delete that file before setting `settings` (or set
`home-manager.backupFileExtension`). There is also `overlays.default`, which adds `pkgs.downer`.

## Using it

```sh
downer README.md notes.md   # one window per file
downer ~/notes              # a folder: a sidebar with its Markdown files as a tree
```

A folder window opens the folder's README if it has one, and its tree follows files being
added, renamed and deleted. Links to other files in the folder open in the same window.

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
