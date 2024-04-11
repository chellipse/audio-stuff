let
  rust_overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");

  pkgs = import <nixpkgs> { overlays = [ rust_overlay ]; };

  # rust = pkgs.rust-bin.stable.latest.default.override {
    # extensions = [ "rust-src" "rustc-codegen-cranelift-preview" ];
  # };

  rust = pkgs.rust-bin.nightly."2024-04-02".default.override {
    extensions = [ "rust-src" "rustc-codegen-cranelift-preview" ];
  };

  # rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
    # extensions = [ "rust-src" "rustc-codegen-cranelift-preview" ];
  # });

in
  pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
    gcc

    rust
    rust-analyzer

    ### dep ###
    openssl
    pkg-config

    libpulseaudio

    libxkbcommon
    xorg.libX11
    xorg.libXcursor
  ];

  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
