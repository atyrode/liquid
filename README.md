# NixOS For Raspberry Pi 3 A+

This builds 64-bit NixOS SD images for a Raspberry Pi 3 A+.

- `pi3a-image` is the small headless rescue image. It boots onto Wi-Fi and starts SSH immediately.
- `pi3a-gui-image` adds a lightweight local maintenance GUI for HDMI, mouse, keyboard, Wi-Fi setup, and Bluetooth setup.

Both images enable the onboard Raspberry Pi 3 Bluetooth controller and install the BlueZ CLI tools.

The headless image uses the `wifi-secrets.conf` workflow below so it can join Wi-Fi without a screen. The GUI image uses NetworkManager instead, so Wi-Fi can be selected on-screen with the network tray applet.

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
gh workflow run build-image.yml --repo atyrode/liquid -f image=pi3a-gui-image -f release_tag=pi3a-gui-lite-$(date +%F)
```

Then download from the latest release, reassemble split parts if needed, and verify the checksum:

```sh
scripts/download-image.sh
```

To download a specific release instead of the latest:

```sh
scripts/download-image.sh --tag pi3a-gui-lite-2026-05-25
```

GitHub Release assets have a 2 GiB per-file limit, so large images may be published as `*.part-*` files. The download script handles that automatically.

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

If you downloaded a release image with `scripts/download-image.sh`, flash that file instead:

```sh
zstd -dc dist/*.img.zst | sudo dd of=/dev/sdX bs=4M conv=fsync status=progress
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

If you flashed the GUI image, plug in HDMI plus a USB mouse and keyboard before boot. It starts greeterless LightDM, logs in as `alex`, opens an IceWM session, and launches a terminal plus tray applets for Wi-Fi and Bluetooth. Use the NetworkManager tray applet to join Wi-Fi and the Blueman tray applet to pair Bluetooth devices.

If the GUI does not start, the first local console also auto-logs in as `alex` so you can inspect logs from the keyboard:

```sh
journalctl -b -u display-manager
```

The GUI variant is intentionally small because the board only has 512 MB RAM and this Pi is meant for installation/setup work, not as a desktop workstation. It does not include XFCE, GNOME, KDE, a browser, or a file manager. It does include graphical Wi-Fi and Bluetooth applets because those are useful for physical setup.

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
