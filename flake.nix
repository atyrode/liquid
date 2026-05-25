{
  description = "NixOS images for Raspberry Pi 3 A+";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      sdImageModule = "${nixpkgs}/nixos/modules/installer/sd-card/sd-image-aarch64.nix";
      pi3aModules = [
        sdImageModule
        ./nixos/pi3a.nix
      ];
      crossModule = {
        nixpkgs.buildPlatform = "x86_64-linux";
        nixpkgs.hostPlatform = "aarch64-linux";
      };
    in
    {
      nixosConfigurations.pi3a = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        modules = pi3aModules;
      };

      nixosConfigurations.pi3a-cross = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = pi3aModules ++ [ crossModule ];
      };

      nixosConfigurations.pi3a-gui = nixpkgs.lib.nixosSystem {
        system = "aarch64-linux";
        modules = pi3aModules ++ [ ./nixos/gui.nix ];
      };

      nixosConfigurations.pi3a-gui-cross = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = pi3aModules ++ [
          ./nixos/gui.nix
          crossModule
        ];
      };

      packages.aarch64-linux.pi3a-image =
        self.nixosConfigurations.pi3a.config.system.build.sdImage;
      packages.aarch64-linux.pi3a-gui-image =
        self.nixosConfigurations.pi3a-gui.config.system.build.sdImage;
      packages.aarch64-linux.default = self.packages.aarch64-linux.pi3a-image;

      packages.x86_64-linux.pi3a-image =
        self.nixosConfigurations.pi3a-cross.config.system.build.sdImage;
      packages.x86_64-linux.pi3a-image-cross =
        self.nixosConfigurations.pi3a-cross.config.system.build.sdImage;
      packages.x86_64-linux.pi3a-gui-image =
        self.nixosConfigurations.pi3a-gui-cross.config.system.build.sdImage;
      packages.x86_64-linux.pi3a-gui-image-cross =
        self.nixosConfigurations.pi3a-gui-cross.config.system.build.sdImage;
      packages.x86_64-linux.default = self.packages.x86_64-linux.pi3a-image;
    };
}
