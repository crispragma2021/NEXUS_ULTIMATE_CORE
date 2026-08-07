{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup
    pkg-config
    openssl
    alsa-lib
    dbus
    udev
    wayland
    libxkbcommon
    libGL
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    pipewire
    clang
    protobuf
    llvmPackages.libclang.lib
    speechd
    mesa
    xdotool
    glib
    gtk3
    libsoup
    webkitgtk_4_1
    cairo
    pango
    gdk-pixbuf
    atk
    harfbuzz
    libsecret
    bpf-linker
  ];

  shellHook = ''
    export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
    echo "🤖 [NEXUS ROOT SHELL] Entorno unificado activo para todo el Workspace de Rust."
  '';
}
