{
  description = "USB CO2 monitor exporter for Prometheus and Home Assistant";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];

      perSystem =
        { system, ... }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ (import inputs.rust-overlay) ];
          };
        in
        {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "cooyote";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            meta = {
              description = "Read USB CO2 monitors and publish to Prometheus and MQTT";
              homepage = "https://github.com/elmarx/cooyote";
              license = with pkgs.lib.licenses; [
                mit
                asl20
              ];
              mainProgram = "cooyote";
              platforms = pkgs.lib.platforms.linux;
            };
          };

          devShells.default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              rust-bin.stable.latest.default
              bacon
              cargo-nextest
              cargo-edit
              cargo-outdated
              just
              mosquitto
            ];
          };

          formatter = pkgs.nixfmt;

          checks.nixos-module = import ./tests/nixos-module.nix {
            inherit pkgs system;
            inherit (inputs) nixpkgs;
            module = inputs.self.nixosModules.default;
          };
        };

      flake = {
        nixosModules.default =
          { config, lib, ... }:
          {
            imports = [ ./nixos-module.nix ];
            config = lib.mkIf config.services.cooyote.enable {
              nixpkgs.overlays = [ inputs.self.overlays.default ];
            };
          };

        overlays.default = final: _prev: {
          cooyote = inputs.self.packages.${final.stdenv.hostPlatform.system}.default;
        };
      };
    };
}
