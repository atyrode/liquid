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

  # Keep first boot lean. Enable bluetooth later once SSH is stable.
  hardware.bluetooth.enable = false;

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
