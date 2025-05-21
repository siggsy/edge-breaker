{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "nixpkgs";
  };

  outputs =
    { nixpkgs, naersk, ... }:
    let
      systems = [
        "x86_64-linux"
        # "x86_64-darwin"
        # "aarch64-linux"
        # "aarch64-darwin"
      ];
      eachSystem =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
          }
        );
    in
    {
      packages = eachSystem (
        { pkgs, ... }:
        let
          naersk-lib = pkgs.callPackage naersk { };
        in
        {
          default = naersk-lib.buildPackage ./.;
        }
      );

      devShells = eachSystem (
        { pkgs, ... }:
        let
          # Runtime shared libraries have to be loaded somehow
          # I choose to not override LD_LIBRARY_PATH as it
          # can break other applications, and instead rely
          # on PKG_CONFIG_PATH
          #
          # If something breaks at runtime its probably either:
          # - missing runtime library path baked into the compiled binary
          # - runtime library is not compatable with the current system
          #
          # For example, wayland apps need wayland-client library
          # which talks to the wayland server. But, if the
          # server and client protocol versions are incompatable, we
          # are in trouble. To fix this we can specify system nixpkgs
          # version for the wayland-client, but this would only
          # work on NixOS machines.
          #
          # On non-NixOS machines, these libraries have to be installed
          # with their own package managers and have LD_LIBRARY_PATH or 
          # PKG_CONFIG_PATH set accordingly (latter being the better option)
          #
          # Godspeed o7.
          runtimeLibraries = with pkgs; [ ];
        in
        {
          default = pkgs.mkShell {
            # Essential packages
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              rust-analyzer
              pkg-config
              lldb
            ];

            PKG_CONFIG_PATH =
              with builtins;
              concatStringsSep ":" (map (lib: "${lib.dev}/lib/pkgconfig") runtimeLibraries);
            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
        }
      );
    };
}
