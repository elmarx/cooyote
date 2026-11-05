{
  pkgs,
  system,
  nixpkgs,
  module,
}:
let
  evaluate =
    settings:
    (nixpkgs.lib.nixosSystem {
      inherit system;
      modules = [
        module
        { services.cooyote = settings; }
      ];
    }).config;
  disabled = evaluate { };
  defaults = evaluate { enable = true; };
  custom = evaluate {
    enable = true;
    http.port = 9090;
    http.openFirewall = true;
    mqttPasswordFile = "/run/secrets/mqtt";
  };
  mqttOnly = evaluate {
    enable = true;
    http.enable = false;
    http.openFirewall = true;
  };
  service = custom.systemd.services.cooyote;
in
assert !(disabled.systemd.services ? cooyote);
assert defaults.networking.firewall.allowedTCPPorts == [ ];
assert !(defaults.systemd.services.cooyote.environment ? MQTT_PASSWORD_FILE);
assert !(defaults.systemd.services.cooyote.serviceConfig ? LoadCredential);
assert custom.networking.firewall.allowedTCPPorts == [ 9090 ];
assert mqttOnly.networking.firewall.allowedTCPPorts == [ ];
assert service.serviceConfig.LoadCredential == "mqtt-password:/run/secrets/mqtt";
assert service.environment.MQTT_PASSWORD_FILE == "%d/mqtt-password";
pkgs.emptyFile
