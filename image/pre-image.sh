#!/usr/bin/env bash
set -euo pipefail

# rpi-image-gen v2.6.0 writes root=/dev/disk/by-slot/system for image-rpios.
# That custom slot symlink can disappear after root partition expansion, and
# plain /dev/disk/by-label links are not reliable early enough in this image's
# initramfs boot path. Use the generated filesystem UUIDs directly instead.
setup_script="${IGconf_image_assetdir:?}/setup.sh"
test -f "$setup_script"

# shellcheck source=/dev/null
source "${IGconf_image_outputdir:?}/img_uuids"
: "${ROOT_UUID:?}"
: "${BOOT_UUID:?}"

cat > "$setup_script" <<SETUP_EOF
#!/bin/bash
set -eu

ROOT_UUID='$ROOT_UUID'
BOOT_UUID='$BOOT_UUID'
LABEL="\$1"

case "\$LABEL" in
   ROOT)
      case "\$IGconf_image_rootfs_type" in
         ext4)
            cat <<EOF > "\$IMAGEMOUNTPATH/etc/fstab"
UUID=\$ROOT_UUID  /  ext4 rw,relatime,errors=remount-ro,commit=30 0 1
EOF
            ;;
         btrfs)
            cat <<EOF > "\$IMAGEMOUNTPATH/etc/fstab"
UUID=\$ROOT_UUID  /  btrfs defaults 0 0
EOF
            ;;
         *)
            ;;
      esac

      cat <<EOF >> "\$IMAGEMOUNTPATH/etc/fstab"
UUID=\$BOOT_UUID  /boot/firmware  vfat defaults,rw,noatime,errors=remount-ro 0 2
EOF
      ;;
   BOOT)
      sed -i "s|root=\([^ ]*\)|root=UUID=\$ROOT_UUID|" "\$IMAGEMOUNTPATH/cmdline.txt"
      ;;
   *)
      ;;
esac
SETUP_EOF

chmod 0755 "$setup_script"
