let
    rust-overlay = builtins.fetchGit {
        url = "https://github.com/oxalica/rust-overlay";
    };
    pkgs = import <nixpkgs> {
        overlays = [
            (import rust-overlay)
        ];
    };
    rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
pkgs.mkShell rec {
    nativeBuildInputs = with pkgs; [
        rust-toolchain
        pkg-config
    ];
    buildInputs = with pkgs; [
        udev
        alsa-lib
        vulkan-loader
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        libxkbcommon
        wayland
        (python313.override {
            enableGIL = false;
        })
    ];
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
    RUST_BACKTRACE = 1;
    RUST_SRC_PATH = "${rust-toolchain}/lib/rustlib/src/rust/library";
}
