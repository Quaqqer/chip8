{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  buildInputs = with pkgs; [ rustc cargo ];

  LD_LIBRARY_PATH = with pkgs;
    lib.makeLibraryPath [
      libxkbcommon
      wayland
      vulkan-loader
    ];
}
