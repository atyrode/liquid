{
  config,
  lib,
  pkgs,
  ...
}:

{
  nixpkgs.hostPlatform = "aarch64-linux";

  networking.hostName = "dogpi";
  networking.useDHCP = lib.mkDefault true;

  # Pi 3 onboard Bluetooth uses ttyAMA0. Keep the HDMI console, but do not
  # attach a serial console to the UART Bluetooth needs.
  boot.kernelParams = lib.mkForce [ "console=tty0" ];

  # The generic aarch64 SD module marks /boot/firmware as noauto.
  # We mount it so wpa_supplicant can read wifi-secrets.conf from the FAT partition.
  fileSystems."/boot/firmware".options = lib.mkForce [
    "defaults"
    "umask=0077"
  ];

  hardware.enableRedistributableFirmware = true;
  hardware.wirelessRegulatoryDatabase = true;
  boot.zfs.forceImportRoot = false;

  networking.wireless = {
    enable = true;
    interfaces = [ "wlan0" ];
    secretsFile = "/boot/firmware/wifi-secrets.conf";
    networks."Livebox-FF30".pskRaw = "ext:psk_livebox";
  };

  services.openssh = {
    enable = true;
    openFirewall = true;
    settings = {
      PasswordAuthentication = false;
      KbdInteractiveAuthentication = false;
      PermitRootLogin = "prohibit-password";
    };
  };

  users.users.root.openssh.authorizedKeys.keys = [
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDL1gyL2UC36tFXiKwazKlBvp7NXMzUWeYcujPcwAVAh alex@ubuntu-4gb-nbg1-1"
  ];

  services.avahi = {
    enable = true;
    nssmdns4 = true;
    openFirewall = true;
  };

  hardware.bluetooth = {
    enable = true;
    powerOnBoot = true;
  };

  systemd.services = {
    bluetooth.wantedBy = lib.mkAfter [ "multi-user.target" ];

    btattach = {
      description = "Attach Raspberry Pi 3 onboard Bluetooth controller";
      before = [ "bluetooth.service" ];
      after = [ "dev-ttyAMA0.device" ];
      requires = [ "dev-ttyAMA0.device" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.bluez}/bin/btattach -B /dev/ttyAMA0 -P bcm -S 3000000";
        Restart = "on-failure";
        RestartSec = 2;
      };
    };
  };

  zramSwap = {
    enable = true;
    memoryPercent = 50;
  };

  documentation.nixos.enable = false;
  documentation.man.enable = false;

  environment.systemPackages = with pkgs; [
    curl
    gitMinimal
    htop
    iw
    jq
    tmux
    usbutils
    wget
  ];

  sdImage.populateFirmwareCommands = lib.mkAfter ''
    cp ${../firmware/wifi-secrets.conf.example} firmware/wifi-secrets.conf.example
  '';

  system.stateVersion = "26.05";
}
