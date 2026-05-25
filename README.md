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

Recommended headless build if you are on a Raspberry Pi/aarch64 machine, or on x86_64 with QEMU binfmt enabled:

```sh
nix build path:$PWD#pi3a-image
```

Build the GUI image instead when you want HDMI, mouse, and keyboard:

```sh
nix build path:$PWD#pi3a-gui-image
```

If this x86_64 machine has QEMU binfmt registered but Nix still refuses `aarch64-linux`, use:

```sh
nix build path:$PWD#pi3a-image --option extra-platforms aarch64-linux
```

For the GUI image with the same workaround:

```sh
nix build path:$PWD#pi3a-gui-image --option extra-platforms aarch64-linux
```

There is also a pure x86_64 cross output, but it can compile a lot from source and is not the fast path:

```sh
nix build path:$PWD#pi3a-image-cross
```

The GUI cross output is:

```sh
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
