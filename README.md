<h1 align="center">Downer</h1>

<p align="center">A small GTK 4 Markdown viewer for Hyprland. Or it will be, one day.</p>

> [!WARNING]
> **Certified, artisanal, vibe-coded slop.**
>
> No human has read this code. An AI writes all of it while a person says "yeah, looks
> good" and presses Enter a lot. Right now it opens an empty window, which is honestly
> its most reliable feature. If you need to read Markdown, any text editor will do a
> better job.
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

## Development

What Downer does is written down in [`spec/`](spec/README.md): every behaviour is a
numbered requirement, and the code follows it. `cargo test` checks the spec's tables
against the code, and `nix flake check` also runs clippy and rustfmt.

## License

MIT, see [LICENSE](LICENSE).
