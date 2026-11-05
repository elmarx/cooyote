{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.cooyote;
  tomlFormat = pkgs.formats.toml { };
  settings = {
    http = {
      enabled = cfg.http.enable;
      port = cfg.http.port;
    };
    mqtt = filterAttrs (_: value: value != null) {
      enabled = cfg.mqtt.enable;
      host = cfg.mqtt.host;
      port = cfg.mqtt.port;
      username = cfg.mqtt.username;
      topic_prefix = cfg.mqtt.topicPrefix;
      availability_topic = cfg.mqtt.availabilityTopic;
      homeassistant_topic = cfg.mqtt.homeassistantTopic;
      discovery_expire = cfg.mqtt.discoveryExpire;
    };
  };
  configFile = tomlFormat.generate "cooyote-config.toml" settings;
in
{
  options.services.cooyote = {
    enable = mkEnableOption "Cooyote USB CO2 monitor exporter";

    package = mkOption {
      type = types.package;
      default = pkgs.cooyote;
      defaultText = literalExpression "pkgs.cooyote";
      description = "The Cooyote package to use.";
    };

    http = {
      enable = mkOption {
        type = types.bool;
        default = true;
        description = "Whether to serve the Prometheus /metrics endpoint.";
      };

      port = mkOption {
        type = types.port;
        default = 8080;
        description = "Port for the Prometheus HTTP endpoint.";
      };

      openFirewall = mkOption {
        type = types.bool;
        default = false;
        description = "Whether to open the HTTP port in the firewall.";
      };
    };

    mqtt = {
      enable = mkEnableOption "MQTT publishing and Home Assistant discovery";

      host = mkOption {
        type = types.str;
        default = "localhost";
        description = "MQTT broker host name or IP address.";
      };

      port = mkOption {
        type = types.port;
        default = 1883;
        description = "MQTT broker port.";
      };

      username = mkOption {
        type = types.nullOr types.str;
        default = null;
        description = "MQTT username.";
      };

      topicPrefix = mkOption {
        type = types.str;
        default = "cooyote";
        description = "Prefix for published MQTT state topics.";
      };

      availabilityTopic = mkOption {
        type = types.str;
        default = "cooyote/availability";
        description = "MQTT topic used to publish online/offline availability.";
      };

      homeassistantTopic = mkOption {
        type = types.str;
        default = "homeassistant";
        description = "Prefix for Home Assistant discovery topics.";
      };

      discoveryExpire = mkOption {
        type = types.str;
        default = "7d";
        example = "12h";
        description = "Lifetime of retained discovery messages in humantime format.";
      };
    };

    mqttPasswordFile = mkOption {
      type = types.nullOr types.str;
      default = null;
      example = "/run/agenix/cooyote-mqtt-password";
      description = ''
        Absolute path to a file containing the MQTT password. Use a runtime
        path, not a Nix path literal, to keep the secret out of the Nix store.
        The file is loaded with systemd's LoadCredential and passed to Cooyote
        through MQTT_PASSWORD_FILE.
      '';
    };

    extraGroups = mkOption {
      type = types.listOf types.str;
      default = [ ];
      description = "Additional groups for the dynamic service user.";
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.mqttPasswordFile == null || hasPrefix "/" cfg.mqttPasswordFile;
        message = "services.cooyote.mqttPasswordFile must be an absolute path.";
      }
    ];

    users.groups.cooyote-device = { };

    services.udev.extraRules = ''
      SUBSYSTEM=="hidraw", ATTRS{idVendor}=="04d9", ATTRS{idProduct}=="a052", GROUP="cooyote-device", MODE="0660", SYMLINK+="cooyote", TAG+="systemd", ENV{SYSTEMD_WANTS}+="cooyote.service"
    '';

    networking.firewall.allowedTCPPorts = optional (
      cfg.http.enable && cfg.http.openFirewall
    ) cfg.http.port;

    systemd.services.cooyote = {
      description = "Cooyote USB CO2 Monitor Exporter";
      # Started by udev (SYSTEMD_WANTS) on plug-in, stopped on removal.
      bindsTo = [ "dev-cooyote.device" ];
      wants = [ "network-online.target" ];
      after = [
        "dev-cooyote.device"
        "network-online.target"
      ];

      environment = {
        COOYOTE_CONFIG = toString configFile;
        MQTT_PASSWORD_FILE = mkIf (cfg.mqttPasswordFile != null) "%d/mqtt-password";
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/cooyote";
        Restart = "on-failure";
        RestartSec = "5s";
        DynamicUser = true;
        SupplementaryGroups = [ "cooyote-device" ] ++ cfg.extraGroups;
        LoadCredential = mkIf (cfg.mqttPasswordFile != null) "mqtt-password:${cfg.mqttPasswordFile}";

        ProtectKernelTunables = true;
        ProtectKernelModules = true;
        ProtectControlGroups = true;
        RestrictAddressFamilies = [
          "AF_UNIX"
          "AF_INET"
          "AF_INET6"
        ];
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        UMask = "0077";
        DevicePolicy = "closed";
        DeviceAllow = [ "char-hidraw rw" ];
      };
    };
  };
}
