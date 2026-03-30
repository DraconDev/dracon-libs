{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Rust toolchain
            cargo
            rustc
            rustfmt
            clippy
            rust-analyzer

            # System libraries for media/tts tools
            alsa-lib
            alsa-lib.dev
            pkg-config
          ];

          buildInputs = with pkgs; [
            # Runtime libraries
            sqlite
          ];

          shellHook = ''
            echo "Dracon libs dev shell loaded"
            export PKG_CONFIG_PATH="${pkgs.alsa-lib.dev}/lib/pkgconfig:$PKG_CONFIG_PATH"
            export LD_LIBRARY_PATH="${pkgs.sqlite.out}/lib:$LD_LIBRARY_PATH"
          '';
        };
      });
}
