# Liquid

Liquid builds a small Raspberry Pi OS Lite image for a Raspberry Pi 3 A+.

The image is intentionally headless:

- Raspberry Pi OS Lite 64-bit base, built with `rpi-image-gen`
- no desktop or GUI
- no repo-managed Wi-Fi passwords
- SSH, Wi-Fi tools, Bluetooth tools, Avahi/mDNS, and diagnostics included
- the Liquid repo is baked into `/home/artist/liquid`
- the terminal renderer starts detached in tmux
- the runtime has one user-facing entrypoint: `liquid`
- local renderer settings live inside the checkout at
  `/home/artist/liquid/.liquid/settings.env`

## Documentation

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): Pi image, single-command
  runtime, local settings, and renderer source layout.
- [docs/DEVELOPMENT_STEPS.md](docs/DEVELOPMENT_STEPS.md): current milestones,
  validation checkpoints, and next operator steps.

## What Is Baked In

The custom image includes:

- `openssh-server`
- `network-manager` and `nmtui`
- `avahi-daemon`
- `bluez`
- `pi-bluetooth`
- `rfkill`
- `iw`
- `raspi-config`
- `git`, `gh`, `build-essential`, `pkg-config`
- `cargo`, `rustc`, `rustfmt`, `rust-clippy`
- `zsh`, Oh My Zsh, `zsh-autosuggestions`, `zsh-syntax-highlighting`
- `tree`, `btop`, `bat`, `dialog`, `fd-find`, `fastfetch`, `fzf`, `nano`,
  `ripgrep`, `tmux`, `zoxide`
- Python basics: `python3`, `python3-venv`, `python3-pip`, `python3-pil`
- a Git checkout of `github.com/atyrode/liquid`
- a prebuilt terminal renderer from `code/examples/terminal.rs`
- the `liquid` command, which dispatches to scripts in `~/liquid`
- local renderer defaults in `~/liquid/.liquid/settings.env`
- `liquid-grow-rootfs`

The shell setup ports the portable parts of `atyrode/nix-dotfiles`: Oh My Zsh,
aliases, `zoxide`, `fzf`, tmux helpers, Git helper, Python venv helpers, and a
non-Nix `zconf` that restarts the login shell. Nix/Home Manager rebuild logic is
not baked in because the Pi image does not use Nix as its system configuration
manager.

The `ls` alias uses `tree -L 1 --noreport --charset utf-8` by default so the
directory connectors stay as line-drawing characters even if the first login
environment has a conservative locale. Set `TREE_CHARSET=ascii` to opt back into
plain ASCII connectors.

When the repo checkout exists at `~/liquid`, the shell loader prefers shell
modules from `~/liquid/image/files/home/artist/.liquid-shell.d`. That lets
normal repo updates change aliases and helpers without a second dotfiles copy.

The installed `/usr/local/bin/liquid` command is only a thin shim into
`~/liquid/scripts/liquid`; the runtime logic lives in the repo checkout.

The image does not include Wi-Fi credentials, SSH private keys, Bluetooth pairing
secrets, or a desktop environment.

## First Boot

Flash the latest Liquid image, boot the Pi with HDMI and a USB keyboard, then log
in on the local console:

```text
user: artist
password: none; tty1 auto-login is enabled
```

The image defaults to a US console keyboard layout. If punctuation keys such as
`~` do not match your physical keyboard, change the layout locally with:

```sh
sudo raspi-config
```

On first boot, the image may reboot once while `liquid-grow-rootfs` expands the
root partition to fill the flashed card. The image boots by the generated root
filesystem UUID instead of `/dev/disk/by-slot/system`, so normal partition
expansion should not break root device discovery.

After the expansion reboot, tty1 auto-logs in as `artist`. The renderer starts
detached in a tmux session named `liquid` using the baked repo checkout at:

```text
/home/artist/liquid
```

Open the runtime menu:

```sh
liquid
```

The menu exposes setup, start, attach, restart, stop, update, Bluetooth, and
diagnostics. Use direct subcommands only when scripting or debugging, for
example `liquid setup`, `liquid restart`, or `liquid attach`.

The setup screen starts the renderer immediately if you press Enter. Move
through values with the arrow keys, use left/right to adjust them, and choose
`Save + start` to write `~/liquid/.liquid/settings.env` before launching. The
default runtime settings are 500 particles, 60 FPS, auto-size on, deep-blue
color, classic charset, and no changing status line.

The renderer hides the changing status line by default to avoid flicker in tmux
and SSH terminals. Enable it only when debugging with `--status` or
`LIQUID_STATUS=1`.

The image does not automatically pull from GitHub on boot. That keeps an
installation from changing behavior just because the network is available.
Update intentionally after Wi-Fi is connected:

```sh
liquid update
```

On a fresh image, these pieces are already baked in:

- `/usr/local/bin/liquid`
- `liquid-renderer.service`
- zsh/Oh My Zsh shell loader files
- the repo checkout at `~/liquid`
- the prebuilt terminal renderer

There is no extra systemd or command installation step after flashing.

On an older already-flashed Pi, migrate once from the repo checkout:

```sh
cd ~/liquid
git pull --ff-only
scripts/liquid install-system
zconf
liquid
```

`install-system` prints the system changes it will make and asks before changing
anything. It installs the single command shim, writes the renderer systemd unit,
loads the repo-backed shell setup, removes the old split helper commands, and
removes `~/liquid-control`. It keeps Wi-Fi, Bluetooth pairings, SSH setup, the
Git checkout, and local renderer settings.

There is no baked password. Set your own password locally before relying on SSH
password login:

```sh
sudo passwd artist
```

The local console auto-login is intentional for recovery/setup. Do not leave the
image unattended on an untrusted physical network before setting your own
password and access policy.

Join Wi-Fi locally:

```sh
sudo nmtui
```

or:

```sh
sudo raspi-config
```

Wi-Fi credentials are intentionally not committed to this repo. For a nearly
zero-touch flash, use Raspberry Pi Imager's OS customization to write the Wi-Fi
network and password at flash time, or add them locally on the Pi with `nmtui`.
Keep those values out of git and out of shared terminal output.

After Wi-Fi is connected, SSH from your workstation:

```sh
ssh artist@dogpi.local
```

If mDNS does not resolve, use the IP shown by `hostname -I` on the Pi or by your
router.

### SSH Host Keys After Reflashing

Every fresh flash generates a fresh SSH host identity on the Pi. Your workstation
may then refuse `ssh artist@dogpi.local` because it remembers the previous Pi
identity for the same hostname. That warning is expected after reflashing.

The safe local fix is to forget the old key before reconnecting:

```sh
scripts/forget-pi-host-key.sh
ssh artist@dogpi.local
```

If you previously connected by IP address, remove that entry too:

```sh
scripts/forget-pi-host-key.sh dogpi.local 192.168.1.42
```

For a disposable trusted LAN only, you can opt out of host-key persistence for
this one host in `~/.ssh/config`:

```sshconfig
Host dogpi.local dogpi
  User artist
  StrictHostKeyChecking no
  UserKnownHostsFile /dev/null
```

That convenience disables SSH's normal man-in-the-middle protection for this
host. The image does not bake static SSH host private keys because that would
give every public image flash the same server identity.

## Download A Release Image

Download the latest public GitHub release without `gh`:

```sh
scripts/download-image.sh
```

The script downloads the latest `liquid-rpi3-lite` image, reassembles split
release assets when needed, and verifies the published SHA256 checksum.

Flash with Raspberry Pi Imager by choosing **Use Custom**, or flash from a
macOS terminal.

List external disks:

```sh
scripts/flash-sd-card-macos.sh --list
```

Then flash the whole SD card disk, not a partition:

```sh
scripts/flash-sd-card-macos.sh --disk /dev/diskN
```

The flashing script destroys the selected disk. It refuses internal disks when
macOS reports them as internal, shows the target disk, and asks you to type the
disk id before writing. It requires `zstd` for `.img.zst` images.

## Bluetooth

Pair a Bluetooth keyboard or device with the image's terminal UI wrapper:

```sh
liquid bluetooth
```

The wrapper uses `dialog` and `bluetoothctl` to scan, select, pair, trust, and
connect a device. If a keyboard passkey is shown, type it on the Bluetooth
keyboard and press Enter.

The raw `bluetoothctl` tool is still available when manual debugging is needed:

```sh
bluetoothctl
```

Bluetooth "ready" means the onboard Pi controller is powered and pairable. The
repo does not pre-pair devices.

## Diagnostics

When Wi-Fi, SSH, or Bluetooth is not behaving, run:

```sh
liquid doctor
```

The output is designed for troubleshooting. Review it before sharing because it
can include local hostnames, IP addresses, and hardware identifiers.

## Build On GitHub

The `Build image` workflow builds and publishes the image on pushes to `main`
that change `code/**`, `image/**`, the Pi scripts, or the workflow itself.

The workflow pins `rpi-image-gen` to `v2.6.0` for reproducible builds.
CI caches that pinned `rpi-image-gen` checkout between runs; it does not cache
generated image work directories or release assets.

The repo includes `image/pre-image.sh`, which patches the pinned
`image-rpios` setup script during image generation so `/etc/fstab` and
`cmdline.txt` use generated filesystem UUIDs instead of slot or label symlinks.
This avoids root-device boot failures from missing `/dev/disk/...` symlinks.

Run it manually from GitHub Actions when needed. Optionally provide a release tag
such as:

```text
v0.1.0
```

Release assets are generated by CI and are not committed to git.

## Local Development

CI checks the source scripts and the baked copies:

```sh
bash -n scripts/liquid
bash -n scripts/*.sh
bash -n image/pre-image.sh
bash -n image/files/usr/local/bin/liquid
bash -n image/files/usr/local/sbin/liquid-grow-rootfs
zsh -n image/files/home/artist/.zshrc image/files/home/artist/.liquid-shell.zsh image/files/home/artist/.liquid-shell.d/*.zsh
cargo check --manifest-path code/Cargo.toml --no-default-features --example terminal
shellcheck scripts/liquid scripts/*.sh image/pre-image.sh image/files/usr/local/bin/liquid image/files/usr/local/sbin/liquid-*
```

`shellcheck` is optional locally, but CI runs it when available.

Run the terminal renderer without touching the main windowed simulation:

```sh
scripts/liquid run --auto-size
```

Choose a color theme with `--color`:

```sh
scripts/liquid run --auto-size --color deep-blue
```

The renderer uses standard ANSI foreground colors for these themes so they work
on the Pi console, SSH, and tmux without requiring truecolor support.

Choose density characters with `--charset`:

```sh
scripts/liquid run --auto-size --charset dots
scripts/liquid run --auto-size --charset blocks
scripts/liquid run --auto-size --charset solid
```

`classic` keeps the current ASCII ramp, `dots` uses dotted density marks,
`blocks` uses shaded block cells, and `solid` uses full-block cells for a
denser water shape.

Adjust the rotating gravity speed with `--gravity-spin`:

```sh
scripts/liquid run --auto-size --gravity-spin 3
```

Open the setup screen explicitly with the repo script:

```sh
scripts/liquid setup
```

For a bounded smoke test that exits on its own:

```sh
scripts/liquid run --fixed-size --cols 40 --rows 20 --particles 500 --color cyan --charset dots --gravity-spin 0 --frames 5
```

Run the windowed developer renderer on a machine with a display:

```sh
cargo run --features window
```
