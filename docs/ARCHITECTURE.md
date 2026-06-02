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

The `.liquid/` directory is ignored by git. The setup UI in `scripts/liquid`
writes this file, and normal `git pull` updates do not replace it.

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

`code/src/raster.rs` owns shared density-grid rasterization for renderers that
turn particle positions into low-resolution pixel cells.

`code/src/terminal.rs` owns the terminal renderer, settings loading, terminal
character/color presentation, and terminal lifecycle. `scripts/liquid` owns the
runtime menu, setup UI, and local settings writes for both terminal and LED
matrix renderers.

`code/src/led_matrix.rs` owns the optional WS2812B LED matrix renderer and
hardware test patterns. It is compiled only with the `led-matrix` feature and
uses SPI0 through Raspberry Pi peripheral access.

`code/examples/terminal.rs` is intentionally thin:

```rust
fn main() -> Result<(), String> {
    fluid_sim::terminal::run_from_env()
}
```

`code/examples/led_matrix.rs` is also thin and exists so the Pi can build the
LED renderer separately from the default terminal renderer.

`code/src/main.rs` remains the windowed developer renderer and uses the shared
library module through `fluid_sim::particle::Particles`.

## Terminal Rendering

The terminal renderer maps the simulation world into a fixed or auto-sized grid,
accumulates nearby particles into density cells, and converts those cells to
terminal characters plus optional color. Selectable character sets keep the
current ASCII ramp available while also supporting dotted and full-cell block
rendering. Color themes use standard ANSI foreground colors instead of 24-bit
RGB so the Pi console, SSH terminals, and tmux do not misinterpret color
sequences as background colors.

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

## LED Matrix Rendering

The LED matrix path is opt-in. `scripts/liquid` builds
`code/examples/led_matrix.rs` with `--no-default-features --features led-matrix`
for `liquid led-test`, `liquid led-orbit`, and `liquid run-led`.

The first supported hardware target is an 8x8 WS2812B serpentine panel on a
Raspberry Pi 3 A+ using SPI0 MOSI. The renderer uses `rppal` for the Pi SPI bus,
`ws2812-spi`'s hosted writer for Linux SBC timing, and `smart-leds` for
brightness/gamma handling.

The LED renderer maps the shared density grid to LED colors, then maps display
coordinates into physical LED indices with configurable origin and linear or
serpentine row order. The defaults match the known Arduino/FastLED test mapping:
row-major indexing with odd rows reversed.

LED settings are stored as additional non-secret values in
`~/liquid/.liquid/settings.env` when the default file is created. Existing
settings files continue to work because `scripts/liquid` supplies defaults when
the LED-specific values are absent. The setup UI edits both renderer families
from the same shared local settings file.
