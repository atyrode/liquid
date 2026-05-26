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

The operator-facing control directory is:

```text
/home/artist/liquid-control
```

`settings.env` in that directory is local installation state. It is intentionally
not replaced by runtime syncs because it contains the current renderer defaults
for that Pi.

The command wrappers in `~/liquid-control` are repo-owned. New images symlink
these files back to:

```text
~/liquid/image/files/home/artist/liquid-control
```

That lets `git pull` update `start`, `restart`, `stop`, `attach`, `config`,
`update`, `doctor`, and `bluetooth` without reflashing. Older flashes can switch
to this model with:

```sh
~/liquid/scripts/sync-pi-runtime.sh
```

The `/usr/local/bin/liquid-*` runtime commands are installed into the image and
can also be refreshed on an existing Pi by the same sync script.

## Boot Behavior

The renderer starts detached in a tmux session named `liquid` through the
`liquid-renderer.service` systemd unit. The unit only starts when the prebuilt
terminal example exists.

The image does not auto-pull from GitHub on boot. Runtime updates are explicit:

```sh
~/liquid-control/update
```

After changing settings, restart the detached renderer with:

```sh
~/liquid-control/restart
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
- hide the cursor for the render loop
- use raw mode so `q`, Esc, and Ctrl-C can exit cleanly
- repaint from `(0, 0)` every frame
- clear the screen when auto-size dimensions change
- restore the cursor, color state, raw mode, and normal screen on exit

This avoids the visible prompt/cursor artifacts caused by repeatedly printing
full frames into the normal terminal scrollback.
