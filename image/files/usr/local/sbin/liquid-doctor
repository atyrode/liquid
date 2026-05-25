#!/usr/bin/env bash
set -uo pipefail

section() {
  printf '\n== %s ==\n' "$1"
}

run_optional() {
  printf '+'
  printf ' %q' "$@"
  printf '\n'
  "$@" 2>&1
  status="$?"
  if [ "$status" -ne 0 ]; then
    echo "(exit $status)"
  fi
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

unit_status() {
  unit="$1"
  if systemctl list-unit-files "${unit}.service" --no-legend 2>/dev/null \
    | awk '{ print $1 }' \
    | grep -qx "${unit}.service"; then
    printf '%-16s enabled=%-10s active=%s\n' \
      "$unit" \
      "$(systemctl is-enabled "$unit" 2>/dev/null || true)" \
      "$(systemctl is-active "$unit" 2>/dev/null || true)"
  else
    printf '%-16s missing\n' "$unit"
  fi
}

echo "Liquid Pi doctor"
echo "Review this output before sharing; it can include local hostnames, IPs, and hardware identifiers."

section "System"
if [ -r /proc/device-tree/model ]; then
  printf 'model: '
  tr -d '\0' </proc/device-tree/model
  printf '\n'
fi
if [ -r /etc/os-release ]; then
  pretty_name="$(
    sed -n 's/^PRETTY_NAME=//p' /etc/os-release \
      | head -n 1 \
      | sed 's/^"//; s/"$//'
  )"
  printf 'os: %s\n' "${pretty_name:-unknown}"
fi
run_optional uname -a
run_optional hostname
run_optional hostname -I

section "Services"
if command_exists systemctl; then
  unit_status ssh
  unit_status NetworkManager
  unit_status avahi-daemon
  unit_status bluetooth
  unit_status hciuart
else
  echo "systemctl missing"
fi

section "Wi-Fi"
if command_exists rfkill; then
  run_optional rfkill list
else
  echo "rfkill missing"
fi
if command_exists nmcli; then
  run_optional nmcli radio all
  run_optional nmcli -f DEVICE,TYPE,STATE device status
else
  echo "nmcli missing"
fi
if command_exists iw; then
  run_optional iw dev
  run_optional iw reg get
else
  echo "iw missing"
fi
if command_exists ip; then
  run_optional ip -brief link
  run_optional ip -brief address
else
  echo "ip missing"
fi

section "Bluetooth"
if command_exists bluetoothctl; then
  run_optional bluetoothctl show
  run_optional bluetoothctl list
else
  echo "bluetoothctl missing"
fi
if command_exists btmgmt; then
  run_optional btmgmt info
else
  echo "btmgmt missing"
fi
if command_exists hciconfig; then
  run_optional hciconfig -a
else
  echo "hciconfig missing"
fi

section "Recent Kernel Hardware Logs"
if command_exists journalctl; then
  journalctl -b -k --no-pager 2>/dev/null \
    | grep -Ei 'brcm|brcmfmac|firmware|wlan|wifi|bluetooth|hci|rfkill' \
    | tail -n 120 || true
else
  echo "journalctl missing"
fi

section "Recent Service Logs"
if command_exists journalctl; then
  run_optional journalctl -b --no-pager -n 100 \
    -u ssh \
    -u NetworkManager \
    -u avahi-daemon \
    -u bluetooth \
    -u hciuart
else
  echo "journalctl missing"
fi
