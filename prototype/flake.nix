# PROTOTYPE (throwaway): dev shell for the cxx-qt + libmpv spike.
{
  inputs.nixpkgs.url = "flake:nixpkgs";
  outputs = { nixpkgs, ... }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      qt = pkgs.qt6;
    in {
      devShells.x86_64-linux.default = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.pkg-config pkgs.cargo pkgs.rustc pkgs.clang pkgs.xvfb-run ];
        buildInputs = [ qt.qtbase qt.qtsvg pkgs.mpv-unwrapped pkgs.libGL pkgs.mesa ];
        shellHook = ''
          export QT_PLUGIN_PATH="${qt.qtbase}/${qt.qtbase.qtPluginPrefix}:${qt.qtsvg}/${qt.qtbase.qtPluginPrefix}"
          export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
          export PROTO_UIC="${qt.qtbase}/libexec/uic"
          export LIBGL_DRIVERS_PATH="${pkgs.mesa}/lib/dri"
          export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${qt.qtbase}/lib:${pkgs.mpv-unwrapped}/lib:${pkgs.libGL}/lib:${pkgs.mesa}/lib:$LD_LIBRARY_PATH"
        '';
      };
    };
}
