{ pkgs ? import <nixpkgs> {} }:

pkgs.stdenv.mkDerivation rec {
  pname = "antigravity";
  version = "1.23.2";

  src = pkgs.fetchurl {
    url = "https://us-central1-apt.pkg.dev/projects/antigravity-auto-updater-dev/pool/antigravity-debian/antigravity_${version}-1776332190_amd64_d29aa2e214aa69c5a7199fce43624422.deb";
    sha256 = "bdd5f32d26791c36640bd2f713f5ebd6e78fe429c3cc27a72668fda6ad6317a4";
  };

  nativeBuildInputs = with pkgs; [
    dpkg
    autoPatchelfHook
    makeWrapper
    wrapGAppsHook3
  ];

  buildInputs = with pkgs; [
    alsa-lib
    at-spi2-core
    cairo
    curl
    dbus
    expat
    glib
    gtk3
    cups
    libsecret
    libsoup
    xorg.libX11
    xorg.libxcb
    xorg.libXcomposite
    xorg.libXdamage
    xorg.libXext
    xorg.libXfixes
    libxkbcommon
    xorg.libxkbfile
    xorg.libXrandr
    mesa
    nspr
    nss
    openssl
    pango
    systemd
    util-linux
    webkitgtk_4_1
    libdrm
    xorg.libxshmfence
    libglvnd
  ];

  unpackPhase = ''
    dpkg-deb --fsys-tarfile $src | tar -x --no-same-owner --no-same-permissions
  '';

  installPhase = ''
    mkdir -p $out/bin $out/opt
    cp -r usr/share/antigravity $out/opt/Antigravity

    # Hacemos ejecutable el binario principal
    chmod +x $out/opt/Antigravity/bin/antigravity

    # Copiamos accesos directos e íconos para el menú del sistema
    mkdir -p $out/share
    cp -r usr/share/applications $out/share/
    cp -r usr/share/icons $out/share/ || true
    cp -r usr/share/pixmaps $out/share/ || true

    # Corregimos las rutas en los archivos .desktop
    substituteInPlace $out/share/applications/antigravity.desktop \
      --replace "Exec=/usr/share/antigravity/antigravity" "Exec=$out/bin/antigravity"

    # Envolvemos el binario para cargar las dependencias correctas en NixOS
    makeWrapper $out/opt/Antigravity/bin/antigravity $out/bin/antigravity \
      --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath buildInputs}" \
      --append-flags "\''${NIXOS_OZONE_GFX_WORKAROUND:+\''${NIXOS_OZONE_GFX_WORKAROUND}}"
  '';

  meta = with pkgs.lib; {
    description = "An agentic development platform from Google, evolving the IDE into the agent-first era.";
    homepage = "https://antigravity.google/";
    license = licenses.unfree;
    platforms = [ "x86_64-linux" ];
  };
}
