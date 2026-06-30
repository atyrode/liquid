# Development Steps

## Current Foundation

- Raspberry Pi OS Lite image builds from `image/layer/liquid-headless-kit.yaml`.
- The image bakes the repo into `/home/artist/liquid`.
- Root filesystem expansion uses filesystem UUIDs instead of fragile
  `/dev/disk/by-slot` or label paths.
- The runtime has one user-facing command: `liquid`, backed by
  `scripts/liquid`.
- Shared density-grid rasterization lives in `code/src/raster.rs`.
- The terminal renderer is a Rust library module with a thin example entrypoint.
- The optional WS2812B LED matrix renderer builds as a separate example behind
  the `led-matrix` feature.
- Chained WS2812B matrices map as separate physical panels by default, with a
  `--continuous-chain` option for strip-like matrix wiring.
- The renderer supports colors, selectable character sets, gravity spin,
  fixed-size grids, auto-size grids, bounded frame counts for smoke tests, and
  an interactive setup screen.

## Current Runtime Workflow

On the Pi:

```sh
liquid
liquid setup
liquid led-orbit
liquid led-test
liquid run-led
liquid restart
liquid attach
liquid update
```

`liquid update` is interactive on the Pi: it fetches from origin, lets the
operator choose a branch, then pulls and rebuilds. Non-interactive update runs
keep the previous current-branch behavior.

Fresh images bake the single `liquid` command, renderer systemd unit, shell
loader files, repo checkout, and prebuilt renderer. On an older already-flashed
Pi, migrate once from the repo checkout:

```sh
cd ~/liquid
git pull --ff-only
scripts/liquid install-system
zconf
liquid
```

The installer prints what it will change before privileged operations. It
removes the previous split runtime helpers and `~/liquid-control`; the runtime
logic remains in `~/liquid/scripts/liquid`.

## Validation Checkpoints

Run these before opening or merging a renderer/runtime change:

```sh
cargo fmt --manifest-path code/Cargo.toml --check
cargo check --manifest-path code/Cargo.toml --no-default-features --example terminal
cargo check --manifest-path code/Cargo.toml --no-default-features --features led-matrix --example led_matrix
cargo check --manifest-path code/Cargo.toml --features window
bash -n scripts/liquid scripts/*.sh image/pre-image.sh image/files/usr/local/bin/liquid image/files/usr/local/sbin/liquid-*
zsh -n image/files/home/artist/.zshrc image/files/home/artist/.liquid-shell.zsh image/files/home/artist/.liquid-shell.d/*.zsh
git diff --check
```

Run a bounded terminal renderer smoke test:

```sh
scripts/liquid run --fixed-size --cols 40 --rows 20 --particles 500 --color cyan --charset dots --gravity-spin 0 --frames 5
```

Run LED matrix checks only on a wired Pi with an external 5V LED power supply:

```sh
liquid led-orbit
liquid led-test
liquid run-led
```

## Next Work

- Test `liquid led-orbit`, `liquid led-test`, and `liquid run-led` on the first
  8x8 WS2812B matrix.
- Record the confirmed LED origin, row order, and panel chain order in the local
  settings or docs.
- Test `liquid` and `liquid setup` on the Pi console and over SSH.
- Test `liquid restart` after saving setup changes.
- Decide whether the terminal renderer should become its own crate if another
  renderer target appears.
- Improve Bluetooth pairing once the current dialog wrapper failure mode is
  observed on real hardware.
