{ config, pkgs, ... }:

{
  # 🔱 NEXUS ZENITH VAULT - OMEGA-31
  # Este archivo gestiona el arsenal de 31 llaves API para el búnker soberano.

  environment.sessionVariables = {
    # CÉLULA 1: dogperro404
    GEMINI_POOL_C1 = "AIzaSyCe2HS2YXnVVoHLFuTICLBuQpfK1Qh8LBc,AIzaSyA81vo3CEe3tnOZnuwxv7j6hB-oW2WYh48,AIzaSyCbv4hT3alohyVgkre0moJd2ADnsZbPZ1g,AIzaSyDsnUoND3zaJqfxtqxonZHHqZY0lYXl-Kw,AIzaSyDh_LxNGgY3ZHaCDxOEjzoKQXcOF9guJv0,AIzaSyDcmdqRlb9xryma4SzXC8LFcMttKMs0tng,AIzaSyAGWz35_6_qzlvFHALp3ZbTJj6RltDyXqk,AIzaSyDZW7lzTykjgRkGSChMdxYuSmTiHqOZceE,AIzaSyCiO48imjM6ujd2oKcNVLiqT5jB_LH5n-I,AIzaSyREDACTADO_2";

    # CÉLULA 2: lucianiaquino53
    GEMINI_POOL_C2 = "AIzaSyREDACTADO_3,AIzaSyBWc5_YR2t59N9nTlQbOiP7hgADim1-8tI,AIzaSyB3DlxT_JzZ9BeFblj5tVERuHlE0L8TqgI,AIzaSyDziJAwmTqWwYvU-BRPj8LtcRnbNiq8H3c,AIzaSyDt45s1KF7XNZ65VkZLDJw2Soax1Woo1pM,AIzaSyBYzQe6xYZr7rUR4JbZSbStBrLKW4BC_aQ,AIzaSyD2lIMcxgTIDXG7xdi3t-5c5m0MF6RFAso,AIzaSyBoqtppOS0Pevsatbot00OH8dQZbRy2b48,AIzaSyDBNHNBGeyrs3CMP0Owk3DMbjqHgdw9eAQ,AIzaSyD8PjGxiqLrWsv78vXrAmYbewKrAC0g_1c";

    # CÉLULA 3: crispragmatico2021
    GEMINI_POOL_C3 = "AIzaSyCz-cknCQb4lEtumlqJZDNdo3cHI75jmQQ,AIzaSyAKKa9JiUciwd5S5ewT50lyHyA57gx_xjg,AIzaSyC5SeoTMQ9aR0EIvR2qBMI3GDjtRyYG0vs,AIzaSyAVkdnX3RNWCgcdjXaxR_NiRK66JLZbKgY,AIzaSyCeKlgbDJRquauuVy2Pxdsg-dBGxY6o2hw,AIzaSyAI5q1IXIrGdE6iW0Gkv8_DD2_SLyboikk,AIzaSyBOiHBn1nDzOz7_tUxUcpHNVen5FkjFl0U,AIzaSyC9Qipq97R9cYo9JsXPEEz3zS1UW3ISp-g,AIzaSyAD8xEeBJ7zOSGI57Xxd0SheTOT6aboKRk,AIzaSyDfkV4xLiLLgnsfNF6TQ9s0DFx1wDHkjgc";

    # CÉLULA 4: divinemercy6321
    GEMINI_POOL_C4 = "AIzaSyREDACTADO_4";

    # NÚCLEO CRÍTICO
    DEEPSEEK_MASTER = "sk-REDACTADO_DEEPSEEK";
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
