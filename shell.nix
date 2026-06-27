{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config
    pkgs.rustc
    pkgs.cargo
    pkgs.clang
    pkgs.pulseaudio
  ];

  buildInputs = [
    pkgs.pipewire
    pkgs.wayland
    pkgs.libxkbcommon  # winit also commonly needs this
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.pipewire}/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
    export LD_LIBRARY_PATH="${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:$LD_LIBRARY_PATH"
  '';
}