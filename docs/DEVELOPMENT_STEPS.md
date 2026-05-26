# Development Steps

## Current Foundation

- Raspberry Pi OS Lite image builds from `image/layer/liquid-headless-kit.yaml`.
- The image bakes the repo into `/home/artist/liquid`.
- Root filesystem expansion uses filesystem UUIDs instead of fragile
  `/dev/disk/by-slot` or label paths.
- The runtime has one user-facing command: `liquid`, backed by
  `scripts/liquid`.
- The terminal renderer is a Rust library module with a thin example entrypoint.
- The renderer supports colors, gravity spin, fixed-size grids, auto-size grids,
  bounded frame counts for smoke tests, and an interactive setup screen.

## Current Runtime Workflow

On the Pi:

```sh
liquid
liquid setup
liquid restart
liquid attach
liquid update
```

On an older already-flashed Pi that does not yet have `/usr/local/bin/liquid`,
run the repo entrypoint directly:

```sh
cd ~/liquid
git pull --ff-only
scripts/liquid
```

Future images install `/usr/local/bin/liquid` as a thin shim into
`~/liquid/scripts/liquid`; the logic remains in the repo checkout.

## Validation Checkpoints

Run these before opening or merging a renderer/runtime change:

```sh
cargo fmt --manifest-path code/Cargo.toml --check
cargo check --manifest-path code/Cargo.toml --no-default-features --example terminal
cargo check --manifest-path code/Cargo.toml --features window
bash -n scripts/liquid scripts/*.sh image/pre-image.sh image/files/usr/local/bin/liquid image/files/usr/local/sbin/liquid-*
zsh -n image/files/home/artist/.zshrc image/files/home/artist/.liquid-shell.zsh image/files/home/artist/.liquid-shell.d/*.zsh
git diff --check
```

Run a bounded terminal renderer smoke test:

```sh
scripts/liquid run --fixed-size --cols 40 --rows 20 --particles 500 --color cyan --gravity-spin 0 --frames 5
```

## Next Work

- Test `liquid` and `liquid setup` on the Pi console and over SSH.
- Test `liquid restart` after saving setup changes.
- Decide whether the terminal renderer should become its own crate if another
  renderer target appears.
- Improve Bluetooth pairing once the current dialog wrapper failure mode is
  observed on real hardware.
