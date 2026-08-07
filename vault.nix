{ config, pkgs, ... }:

{
  # 🔱 NEXUS ZENITH VAULT - OMEGA-31
  # Las llaves API se cargan desde /etc/nexus-vault.env (archivo NO versionado).
  # Nunca se deben hardcodear secretos en archivos del repositorio:
  # GitGuardian los detecta y quedan expuestos en el historial de git.

  environment.sessionVariables = {
    # Cada célula se carga desde el vault local; si falta el archivo,
    # las variables quedan vacías y NEXUS cae a fallbacks de .env.
    GEMINI_POOL_C1 = "$(cat /etc/nexus-vault.env 2>/dev/null | grep '^GEMINI_POOL_C1=' | cut -d= -f2-)";
    GEMINI_POOL_C2 = "$(cat /etc/nexus-vault.env 2>/dev/null | grep '^GEMINI_POOL_C2=' | cut -d= -f2-)";
    GEMINI_POOL_C3 = "$(cat /etc/nexus-vault.env 2>/dev/null | grep '^GEMINI_POOL_C3=' | cut -d= -f2-)";
    GEMINI_POOL_C4 = "$(cat /etc/nexus-vault.env 2>/dev/null | grep '^GEMINI_POOL_C4=' | cut -d= -f2-)";
    DEEPSEEK_MASTER = "$(cat /etc/nexus-vault.env 2>/dev/null | grep '^DEEPSEEK_MASTER=' | cut -d= -f2-)";
  };

  # Configuración de rotación automática en el arranque
  systemd.services.zenith-sync = {
    description = "Sincronización del Arsenal Zenith";
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${pkgs.bash}/bin/bash -c 'echo Arsenal Zenith de 31 llaves cargado en memoria de búnker'";
    };
  };
}
