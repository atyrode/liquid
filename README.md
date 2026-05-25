# Headless NixOS For Raspberry Pi 3 A+

This builds a 64-bit NixOS SD image for a Raspberry Pi 3 A+ that boots onto Wi-Fi and starts SSH immediately.

The Wi-Fi password is not stored in the Nix files. After flashing, put this file on the first FAT partition of the SD card:

```text
wifi-secrets.conf
```

with this content:

```text
psk_livebox=your-wifi-password
```

Recommended build if you are on a Raspberry Pi/aarch64 machine, or on x86_64 with QEMU binfmt enabled:

```sh
nix build path:$PWD#pi3a-image
```

If this x86_64 machine has QEMU binfmt registered but Nix still refuses `aarch64-linux`, use:

```sh
nix build path:$PWD#pi3a-image --option extra-platforms aarch64-linux
```

There is also a pure x86_64 cross output, but it can compile a lot from source and is not the fast path:

```sh
nix build path:$PWD#pi3a-image-cross
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
