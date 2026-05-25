{
  lib,
  pkgs,
  ...
}:

{
  users.users.alex = {
    isNormalUser = true;
    description = "Alex";
    extraGroups = [
      "audio"
      "input"
      "networkmanager"
      "video"
      "wheel"
    ];
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDL1gyL2UC36tFXiKwazKlBvp7NXMzUWeYcujPcwAVAh alex@ubuntu-4gb-nbg1-1"
    ];
  };

  security.sudo.wheelNeedsPassword = false;

  hardware.graphics.enable = true;

  networking.wireless = {
    enable = lib.mkForce false;
    networks = lib.mkForce { };
  };
  networking.networkmanager = {
    enable = true;
    wifi.backend = "wpa_supplicant";
  };

  services.xserver = {
    enable = true;
    videoDrivers = [ "modesetting" ];
    windowManager.icewm.enable = true;

    # The GUI image is a maintenance surface for local HDMI/USB access, not a
    # desktop workstation. Autostart one terminal and keep the session small.
    displayManager.sessionCommands = ''
      ${pkgs.xsetroot}/bin/xsetroot -solid '#202020'
      ( sleep 2; ${pkgs.networkmanagerapplet}/bin/nm-applet ) &
      ( sleep 2; ${pkgs.blueman}/bin/blueman-applet ) &
      ${pkgs.xterm}/bin/xterm -geometry 120x36+24+24 -fa monospace -fs 12 -title dogpi &
    '';

    displayManager.lightdm = {
      enable = true;
      greeter.enable = false;
    };
  };

  services.displayManager.autoLogin = {
    enable = true;
    user = "alex";
  };

  services.getty.autologinUser = "alex";
  services.blueman.enable = true;

  zramSwap.memoryPercent = lib.mkForce 75;

  environment.systemPackages = with pkgs; [
    blueman
    networkmanagerapplet
    xterm
    xrandr
    xset
    xsetroot
  ];
}
