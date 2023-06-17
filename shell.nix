{pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs; [ cargo rustc pkg-config libudev-zero ];
}
