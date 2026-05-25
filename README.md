# NixOS For Raspberry Pi 3 A+

This builds 64-bit NixOS SD images for a Raspberry Pi 3 A+.

- `pi3a-image` is the small headless rescue image. It boots onto Wi-Fi and starts SSH immediately.
- `pi3a-gui-image` adds a local XFCE desktop for HDMI, mouse, and keyboard control.

Both images enable the onboard Raspberry Pi 3 Bluetooth controller and install the BlueZ CLI tools. The GUI image also includes Blueman for graphical Bluetooth pairing.

The Wi-Fi password is not stored in the Nix files. After flashing, put this file on the first FAT partition of the SD card:

```text
wifi-secrets.conf
```

with this content:

```text
psk_livebox=your-wifi-password
```

## Build On GitHub

Use the GitHub workflow when building from an x86_64 machine. The normal image is an ARM64 NixOS system, so local x86_64 builds either need emulation or a slow cross build.

From the GitHub UI:

1. Open **Actions**.
2. Select **Build image**.
3. Click **Run workflow**.
4. Choose `pi3a-gui-image` or `pi3a-image`.
5. Optionally set a release tag such as `pi3a-gui-2026-05-25`.

If no release tag is set, download the image from the workflow run artifact. If a release tag is set, download it from the repository release assets.

Using `gh`:

```sh
gh workflow run build-image.yml --repo atyrode/liquid -f image=pi3a-gui-image -f release_tag=pi3a-gui-2026-05-25
```

Then download from the release:

```sh
gh release download pi3a-gui-2026-05-25 --repo atyrode/liquid --pattern '*.img.zst*' --dir dist
```

## Local Build

Recommended local headless build if you are on a Raspberry Pi/aarch64 machine:

```sh
nix build path:$PWD#pi3a-image
```

Build the GUI image on a Raspberry Pi/aarch64 machine when you want HDMI, mouse, and keyboard:

```sh
nix build path:$PWD#pi3a-gui-image
```

On x86_64, these package names resolve to the explicit cross outputs:

```sh
nix build path:$PWD#pi3a-image
nix build path:$PWD#pi3a-gui-image
```

The explicit cross output names are also available, but they can compile a lot from source and are not the fast path:

```sh
nix build path:$PWD#pi3a-image-cross
nix build path:$PWD#pi3a-gui-image-cross
```

Flash, replacing `/dev/sdX` with the whole SD card device, not a partition:

```sh
zstd -dc result/sd-image/*.img.zst | sudo dd of=/dev/sdX bs=4M conv=fsync status=progress
sync
```

Unplug/replug the SD card, open the first partition named `FIRMWARE`, and copy `wifi-secrets.conf` there.

Boot the Pi and SSH:

```sh
ssh root@dogpi.local
```

If mDNS does not resolve:

```sh
ssh root@THE_ROUTER_IP
```

The image authorizes the public key from `/home/alex/.ssh/id_ed25519.pub`.

If you flashed the GUI image, plug in HDMI plus a USB mouse and keyboard before boot. It starts LightDM and logs into XFCE as `alex`. If the desktop does not start, the first local console also auto-logs in as `alex` so you can inspect logs from the keyboard:

```sh
journalctl -b -u display-manager
```

The GUI variant is expected to feel modest on a Pi 3 A+ because the board only has 512 MB RAM. It intentionally uses XFCE instead of GNOME or KDE and does not include a browser by default.

Check Bluetooth after SSH:

```sh
systemctl status btattach bluetooth
bluetoothctl list
```

Pair from SSH:

```sh
bluetoothctl
power on
agent on
default-agent
scan on
pair XX:XX:XX:XX:XX:XX
trust XX:XX:XX:XX:XX:XX
connect XX:XX:XX:XX:XX:XX
```

This enables basic Bluetooth pairing/control. Bluetooth audio is intentionally not enabled in the rescue image; it needs PipeWire or PulseAudio and more packages.
