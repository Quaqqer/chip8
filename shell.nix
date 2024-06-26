{
  pkgs ? import <nixpkgs> { },
}:
let
  ld_library_path = pkgs.lib.makeLibraryPath (
    with pkgs;
    [
      libxkbcommon
      wayland
      vulkan-loader
      xorg.libX11
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
    ]
  );
in
pkgs.mkShell {
  packages = with pkgs; [
    pkg-config
    gtk3
    wrapGAppsHook
  ];

  LD_LIBRARY_PATH = ld_library_path;

  shellHook = ''
    export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH"
  '';
}
