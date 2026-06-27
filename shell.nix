{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = [
    pkgs.pkg-config
    pkgs.rustc
    pkgs.cargo
    pkgs.clang
  ];

  buildInputs = [
    pkgs.pipewire
  ];

  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.pipewire}/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
  '';
}