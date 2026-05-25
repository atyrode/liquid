# Liquid

Liquid is now a small operational kit for a Raspberry Pi 3 A+ running stock
Raspberry Pi OS Lite 64-bit.

The goal is deliberately narrow:

- stock Raspberry Pi OS Lite for hardware compatibility
- no custom NixOS image build
- no repo-managed Wi-Fi secrets or generated OS images
- SSH, Wi-Fi, and Bluetooth brought up with simple headless scripts

## Flash The Pi

Use Raspberry Pi Imager on your workstation:

1. Choose **Raspberry Pi OS Lite 64-bit**.
2. Open OS customisation.
3. Set the hostname, for example `dogpi`.
4. Set your user account.
5. Enable SSH with your public key.
6. Do not configure Wi-Fi in Imager for this repo workflow.
7. Flash the microSD card.

The Pi 3 A+ has no Ethernet, so first Wi-Fi setup needs local access. Boot with
HDMI and a USB keyboard, log in, then run one of:

```sh
sudo nmtui
```

or:

```sh
sudo raspi-config
```

Use the tool to join Wi-Fi. After Wi-Fi is connected, SSH from your workstation:

```sh
ssh your-user@dogpi.local
```

If mDNS does not resolve, use the IP shown by `hostname -I` on the Pi or by your
router.

## Bootstrap

Copy the scripts to the Pi and run the bootstrap:

```sh
scp scripts/bootstrap-pi.sh scripts/pi-doctor.sh your-user@dogpi.local:/tmp/
ssh your-user@dogpi.local
bash /tmp/bootstrap-pi.sh
```

The bootstrap asks before making changes. For unattended use after reviewing it:

```sh
bash /tmp/bootstrap-pi.sh --yes
```

It installs and enables only the basics needed for this project:

- SSH server
- NetworkManager CLI/TUI
- Avahi/mDNS
- BlueZ Bluetooth tools
- Raspberry Pi Bluetooth support
- rfkill

## Bluetooth

After bootstrap, pair devices from SSH:

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
bash /tmp/pi-doctor.sh
```

The output is designed for troubleshooting. Review it before sharing because it
can include local hostnames, IP addresses, and hardware identifiers.

## Development

This repository no longer builds or publishes OS images. CI only checks the
operator scripts:

```sh
bash -n scripts/*.sh
shellcheck scripts/*.sh
```

`shellcheck` is optional locally, but CI runs it when available.
