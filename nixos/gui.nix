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
      "video"
      "wheel"
    ];
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIDL1gyL2UC36tFXiKwazKlBvp7NXMzUWeYcujPcwAVAh alex@ubuntu-4gb-nbg1-1"
    ];
  };

  security.sudo.wheelNeedsPassword = false;

  hardware.graphics.enable = true;

  services.xserver = {
    enable = true;
    videoDrivers = [ "modesetting" ];

    desktopManager.xfce = {
      enable = true;
      enableScreensaver = false;
    };

    displayManager.lightdm = {
      enable = true;
      greeters.gtk.enable = true;
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
    xterm
    xfce4-whiskermenu-plugin
  ];
}
