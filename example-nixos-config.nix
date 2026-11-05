{
  description = "Example NixOS configuration with Cooyote";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cooyote.url = "github:elmarx/cooyote";
    cooyote.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, cooyote, ... }: {
    nixosConfigurations.example-host = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        cooyote.nixosModules.default
        {
          services.cooyote = {
            enable = true;
            http.openFirewall = true;
            mqtt = {
              enable = true;
              host = "mqtt.example.com";
              username = "cooyote";
            };
            # Provision separately, e.g. with agenix or sops-nix.
            mqttPasswordFile = "/run/agenix/cooyote-mqtt-password";
          };
        }
        # Add your host's hardware configuration and other NixOS settings here.
      ];
    };
  };
}
