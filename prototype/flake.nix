# PROTOTYPE (throwaway): dev shell for the cxx-qt + libmpv spike.
{
  inputs.nixpkgs.url = "flake:nixpkgs";
  outputs = { nixpkgs, ... }:
    let
      lib = nixpkgs.lib;
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
    in {
      devShells = lib.genAttrs systems (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          qt = pkgs.qt6;
          linux = pkgs.stdenv.hostPlatform.isLinux;
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = [ pkgs.pkg-config pkgs.cargo pkgs.rustc pkgs.clang ]
              ++ lib.optionals linux [ pkgs.xvfb-run ];
            buildInputs = [ qt.qtbase qt.qtsvg pkgs.mpv-unwrapped ]
              ++ lib.optionals linux [ pkgs.libGL pkgs.mesa ];
            shellHook = ''
              export QT_PLUGIN_PATH="${qt.qtbase}/${qt.qtbase.qtPluginPrefix}:${qt.qtsvg}/${qt.qtbase.qtPluginPrefix}"
              export LIBCLANG_PATH="${pkgs.libclang.lib}/lib"
              export PROTO_UIC="${qt.qtbase}/libexec/uic"
            '' + lib.optionalString linux ''
              export LIBGL_DRIVERS_PATH="${pkgs.mesa}/lib/dri"
              export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${qt.qtbase}/lib:${pkgs.mpv-unwrapped}/lib:${pkgs.libGL}/lib:${pkgs.mesa}/lib:$LD_LIBRARY_PATH"
            '' + lib.optionalString (!linux) ''
              export DYLD_FALLBACK_LIBRARY_PATH="${qt.qtbase}/lib:${pkgs.mpv-unwrapped}/lib:$DYLD_FALLBACK_LIBRARY_PATH"
            '';
          };
        });
    };
}
