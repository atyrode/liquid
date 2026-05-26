# Development Steps

## Current Foundation

- Raspberry Pi OS Lite image builds from `image/layer/liquid-headless-kit.yaml`.
- The image bakes the repo into `/home/artist/liquid`.
- Root filesystem expansion uses filesystem UUIDs instead of fragile
  `/dev/disk/by-slot` or label paths.
- Runtime commands and control scripts are tracked in the repo.
- The terminal renderer is a Rust library module with a thin example entrypoint.
- The renderer supports colors, gravity spin, fixed-size grids, auto-size grids,
  bounded frame counts for smoke tests, and an interactive setup screen.

## Current Runtime Workflow

On the Pi:

```sh
~/liquid-control/update
~/liquid-control/config
~/liquid-control/restart
~/liquid-control/attach
```

For an already-flashed Pi that predates repo-owned control symlinks:

```sh
cd ~/liquid
git pull --ff-only
~/liquid/scripts/sync-pi-runtime.sh
zconf
```

`sync-pi-runtime.sh` updates `/usr/local/bin/liquid-*`, links
`~/liquid-control` command scripts back to the repo, and preserves
`~/liquid-control/settings.env`.

## Validation Checkpoints

Run these before opening or merging a renderer/runtime change:

```sh
cargo fmt --manifest-path code/Cargo.toml --check
cargo check --manifest-path code/Cargo.toml --no-default-features --example terminal
cargo check --manifest-path code/Cargo.toml --features window
bash -n scripts/*.sh image/pre-image.sh image/files/home/artist/liquid-control/attach image/files/home/artist/liquid-control/bluetooth image/files/home/artist/liquid-control/config image/files/home/artist/liquid-control/doctor image/files/home/artist/liquid-control/restart image/files/home/artist/liquid-control/start image/files/home/artist/liquid-control/stop image/files/home/artist/liquid-control/update image/files/usr/local/bin/liquid-* image/files/usr/local/sbin/liquid-*
zsh -n image/files/home/artist/.zshrc image/files/home/artist/.liquid-shell.zsh image/files/home/artist/.liquid-shell.d/*.zsh
git diff --check
```

Run a bounded terminal renderer smoke test:

```sh
cd code
cargo run --release --no-default-features --example terminal -- --cols 40 --rows 20 --particles 500 --color cyan --gravity-spin 0 --frames 5
```

## Next Work

- Test `liquid-run-terminal --setup` on the Pi console and over SSH.
- Test `~/liquid-control/restart` after editing `settings.env`.
- Test `~/liquid/scripts/sync-pi-runtime.sh` on an already-flashed image.
- Decide whether the terminal renderer should become its own crate if another
  renderer target appears.
- Improve Bluetooth pairing once the current dialog wrapper failure mode is
  observed on real hardware.
