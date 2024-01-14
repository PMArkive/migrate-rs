{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.taspromto;
in {
  options.services.taspromto = {
    enable = mkEnableOption "taspromto";

    source = mkOption {
      type = types.str;
      default = "https://api.demos.tf";
      description = "Api endpoint to migrate demos for";
    };

    baseUrl = mkOption {
      type = types.str;
      description = "base url the local demos are stored at";
    };

    storageRoot = mkOption {
      type = types.str;
      description = "path local demo files are stored at";
    };

    backend = mkOption {
      type = types.str;
      description = "name of the local demos backend";
    };

    age = mkOption {
      type = types.int;
      default = 172800;
      description = "age of demos to migrate";
    };

    keyFile = mkOption {
      type = types.str;
      description = "path containing the edit secret";
    };

    user = mkOption {
      type = types.str;
      description = "user that owns the local demos";
    };

    package = mkOption {
      type = types.package;
      defaultText = literalExpression "pkgs.demostf-migrate";
      description = "package to use";
    };
  };

  config = mkIf cfg.enable {
    systemd.services."demostf-migrate" = {
      wantedBy = ["multi-user.target"];
      environment = {
        SOURCE = cfg.source;
        BASE_URL = cfg.baseUrl;
        STORAGE_ROOT = cfg.storageRoot;
        BACKEND = cfg.backend;
        AGE = toString cfg.age;
      };

      serviceConfig = {
        ExecStart = "${cfg.package}/bin/demostf-migrate";
        ReadWritePaths = [cfg.storageRoot];
        EnvironmentFile = cfg.keyFile;
        Restart = "on-failure";
        User = cfg.user;
        PrivateTmp = true;
        ProtectSystem = "strict";
        ProtectHome = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        ProtectClock = true;
        CapabilityBoundingSet = true;
        ProtectKernelLogs = true;
        ProtectControlGroups = true;
        SystemCallArchitectures = "native";
        ProtectKernelModules = true;
        RestrictNamespaces = true;
        MemoryDenyWriteExecute = true;
        ProtectHostname = true;
        LockPersonality = true;
        ProtectKernelTunables = true;
        RestrictAddressFamilies = "AF_INET AF_INET6";
        RestrictRealtime = true;
        ProtectProc = "noaccess";
        SystemCallFilter = ["@system-service" "~@resources" "~@privileged"];
        IPAddressDeny = "localhost link-local multicast";
        ProcSubset = "pid";
      };
    };

    systemd.timers."demostf-migrate" = {
      enable = true;
      description = "Migrate demos for demos.tf";
      wantedBy = ["multi-user.target"];
      timerConfig = {
        OnCalendar = "*:0/10";
      };
    };
  };
}
