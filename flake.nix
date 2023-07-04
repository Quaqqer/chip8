{
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.naersk.url = "github:nix-community/naersk";
  inputs.nixpkgs.url = "nixpkgs";

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        lib = pkgs.lib;
        naersk' = pkgs.callPackage naersk { };

        ld_library_path = lib.makeLibraryPath (with pkgs; [
          libxkbcommon
          wayland
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
        ]);

        pkg = naersk'.buildPackage {
          src = ./.;
          buildInputs = with pkgs; [ makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/chip8 --prefix LD_LIBRARY_PATH : ${ld_library_path}
          '';
        };
      in rec {
        defaultPackage = pkg;
        packages.chip8 = pkg;

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo ];
          LD_LIBRARY_PATH = ld_library_path;
        };
      });
}
