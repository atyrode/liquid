{
  description = "Headless NixOS image for Raspberry Pi 3 A+";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    {
      nixosConfigurations.pi3a = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        modules = [
          "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
          ./nixos/pi3a.nix
        ];
      };

      nixosConfigurations.pi3a-cross = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix"
          ./nixos/pi3a.nix
          {
            nixpkgs.buildPlatform = "x86_64-linux";
            nixpkgs.hostPlatform = "aarch64-linux";
          }
        ];
      };

      packages.x86_64-linux.pi3a-image =
        self.nixosConfigurations.pi3a.config.system.build.sdImage;
      packages.x86_64-linux.pi3a-image-cross =
        self.nixosConfigurations.pi3a-cross.config.system.build.sdImage;
      packages.x86_64-linux.default = self.packages.x86_64-linux.pi3a-image;
    };
}
