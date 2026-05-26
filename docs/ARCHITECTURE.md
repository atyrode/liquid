# Architecture

Liquid is two things kept in one repo:

- a Raspberry Pi OS Lite image for the installation runtime
- a Rust simulation/renderer workspace under `code/`

The image is treated as the deployment artifact. The baked checkout at
`/home/artist/liquid` remains a normal Git repo so the Pi can pull runtime
updates after Wi-Fi is configured.

## Pi Runtime Layout

The image creates the `artist` user and bakes the repo into:

```text
/home/artist/liquid
```

The runtime entrypoint inside the checkout is:

```text
/home/artist/liquid/scripts/liquid
```

The installed command is a thin shim:

```text
/usr/local/bin/liquid -> /home/artist/liquid/scripts/liquid
```

All operator actions go through that one command. Running `liquid` without
arguments opens a terminal menu for setup, renderer control, updates, Bluetooth,
and diagnostics. Direct subcommands exist for scriptable actions such as
`liquid setup`, `liquid restart`, `liquid bluetooth`, and `liquid doctor`.

Renderer settings are local installation state stored inside the checkout at:

```text
/home/artist/liquid/.liquid/settings.env
```

The `.liquid/` directory is ignored by git. The setup UI writes this file, and
normal `git pull` updates do not replace it.

Fresh images bake the command shim, systemd unit, shell loader files, repo
checkout, and prebuilt renderer into the filesystem. Older already-flashed Pis
can migrate to the same layout with one repo-owned action:

```sh
cd ~/liquid
git pull --ff-only
scripts/liquid install-system
```

That migration removes the old split helper commands and `~/liquid-control`,
but keeps the Git checkout, local renderer settings, Wi-Fi, Bluetooth pairings,
SSH setup, and other system state.

## Boot Behavior

The renderer starts detached in a tmux session named `liquid` through the
`liquid-renderer.service` systemd unit. The unit only starts when the prebuilt
terminal example exists.

The image does not auto-pull from GitHub on boot. Runtime updates are explicit:

```sh
liquid update
```

After changing settings, restart the detached renderer with:

```sh
liquid restart
```

## Rust Source Layout

`code/src/particle.rs` owns the simulation data model and stepping logic.

`code/src/terminal.rs` owns the terminal renderer, settings loading, setup UI,
terminal rasterization, and terminal lifecycle.

`code/examples/terminal.rs` is intentionally thin:

```rust
fn main() -> Result<(), String> {
    fluid_sim::terminal::run_from_env()
}
```

`code/src/main.rs` remains the windowed developer renderer and uses the shared
library module through `fluid_sim::particle::Particles`.

## Terminal Rendering

The terminal renderer maps the simulation world into a fixed or auto-sized grid,
accumulates nearby particles into density cells, and converts those cells to
terminal characters plus optional color.

Interactive rendering uses crossterm to:

- enter the terminal alternate screen
- disable terminal line-wrap during animation
- hide the cursor for the render loop
- use raw mode so `q`, Esc, and Ctrl-C can exit cleanly
- repaint from `(0, 0)` every frame without printing a trailing newline that can
  scroll the bottom edge
- clear the screen when auto-size dimensions change
- restore the cursor, color state, raw mode, and normal screen on exit

The changing status line is hidden by default because it redraws every frame and
is visually noisy over SSH or tmux. Use `--status` or `LIQUID_STATUS=1` when
debugging frame/config values.

This avoids the visible prompt/cursor artifacts caused by repeatedly printing
full frames into the normal terminal scrollback.
