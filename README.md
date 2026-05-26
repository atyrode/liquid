# Liquid

Liquid builds a small Raspberry Pi OS Lite image for a Raspberry Pi 3 A+.

The image is intentionally headless:

- Raspberry Pi OS Lite 64-bit base, built with `rpi-image-gen`
- no desktop or GUI
- no repo-managed Wi-Fi passwords
- SSH, Wi-Fi tools, Bluetooth tools, Avahi/mDNS, and diagnostics included
- the Liquid repo is baked into `/home/liquid/liquid`
- the terminal renderer starts automatically in tmux on the local console

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
- `zsh`, `zsh-autosuggestions`, `zsh-syntax-highlighting`
- `tree`, `btop`, `bat`, `fd-find`, `fastfetch`, `fzf`, `ripgrep`, `tmux`, `zoxide`
- Python basics: `python3`, `python3-venv`, `python3-pip`, `python3-pil`
- a Git checkout of `github.com/atyrode/liquid`
- a prebuilt terminal renderer from `code/examples/terminal.rs`
- `liquid-bootstrap`
- `liquid-doctor`
- `liquid-grow-rootfs`

The image does not include Wi-Fi credentials, SSH private keys, Bluetooth pairing
secrets, or a desktop environment.

## First Boot

Flash the latest Liquid image, boot the Pi with HDMI and a USB keyboard, then log
in on the local console:

```text
user: liquid
password: none; tty1 auto-login is enabled
```

On first boot, the image may reboot once while `liquid-grow-rootfs` expands the
root partition to fill the flashed card. The image boots by filesystem label
instead of `/dev/disk/by-slot/system`, so normal partition expansion should not
break root device discovery.

After the expansion reboot, tty1 auto-logs in as `liquid` and attaches to a
tmux session named `liquid`. That tmux session runs the prebuilt terminal
renderer using the baked repo checkout at:

```text
/home/liquid/liquid
```

Detach from the renderer with `Ctrl-b` then `d`. Reattach locally or over SSH:

```sh
tmux attach -t liquid
```

Start it manually if needed:

```sh
liquid-start --attach
```

The image does not automatically pull from GitHub on boot. That keeps an
installation from changing behavior just because the network is available.
Update intentionally after Wi-Fi is connected:

```sh
liquid-update
```

There is no baked password. Set your own password locally before relying on SSH
password login:

```sh
sudo passwd liquid
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
ssh liquid@dogpi.local
```

If mDNS does not resolve, use the IP shown by `hostname -I` on the Pi or by your
router.

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

After Wi-Fi and SSH are working, pair devices over SSH:

```sh
bluetoothctl
power on
agent on
default-agent
scan on
```

Put the Bluetooth keyboard or device in pairing mode, then replace the address
below:

```text
pair XX:XX:XX:XX:XX:XX
trust XX:XX:XX:XX:XX:XX
connect XX:XX:XX:XX:XX:XX
quit
```

Bluetooth "ready" means the onboard Pi controller is powered and pairable. The
repo does not pre-pair devices.

## Diagnostics

When Wi-Fi, SSH, or Bluetooth is not behaving, run:

```sh
liquid-doctor
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
`cmdline.txt` use `/dev/disk/by-label/ROOT` and `/dev/disk/by-label/BOOT`
instead of slot symlinks. This avoids the root-device boot failure seen after
root partition expansion.

Run it manually from GitHub Actions when needed. Optionally provide a release tag
such as:

```text
v0.1.0
```

Release assets are generated by CI and are not committed to git.

## Local Development

CI checks the operator scripts and the baked copies:

```sh
bash -n scripts/*.sh
bash -n image/pre-image.sh
bash -n image/files/usr/local/bin/liquid-run-terminal
bash -n image/files/usr/local/bin/liquid-start
bash -n image/files/usr/local/bin/liquid-update
bash -n image/files/usr/local/sbin/liquid-bootstrap
bash -n image/files/usr/local/sbin/liquid-doctor
bash -n image/files/usr/local/sbin/liquid-grow-rootfs
cmp scripts/bootstrap-pi.sh image/files/usr/local/sbin/liquid-bootstrap
cmp scripts/pi-doctor.sh image/files/usr/local/sbin/liquid-doctor
cargo check --manifest-path code/Cargo.toml --no-default-features --example terminal
shellcheck scripts/*.sh image/pre-image.sh image/files/usr/local/bin/liquid-* image/files/usr/local/sbin/liquid-*
```

`shellcheck` is optional locally, but CI runs it when available.

Run the terminal renderer without touching the main windowed simulation:

```sh
cd code
cargo run --release --no-default-features --example terminal -- --auto-size
```

Choose a color theme with `--color`:

```sh
cargo run --release --no-default-features --example terminal -- --auto-size --color deep-blue
```

Adjust the rotating gravity speed with `--gravity-spin`:

```sh
cargo run --release --no-default-features --example terminal -- --auto-size --gravity-spin 3
```

For a bounded smoke test that exits on its own:

```sh
cargo run --release --no-default-features --example terminal -- --cols 40 --rows 20 --particles 500 --color cyan --gravity-spin 0 --frames 5
```

Run the windowed developer renderer on a machine with a display:

```sh
cargo run --features window
```
