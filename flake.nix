{
  description = "Rust package using webrtc crate";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable"; # or unstable if you prefer
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        rustToolchain = pkgs.rustPlatform.rustcSrc;
        rust = pkgs.rustPlatform;

      in
      {
        packages.default = rust.buildRustPackage rec {
          pname = "webrtc-demo";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
            libnice
            libffi
            glib
            livekit-libwebrtc
            libva
            wayland
            libxkbcommon
            pkgs.linuxHeaders
          ];

          # Some crates use system SSL paths or need environment hints
          RUSTFLAGS = "-C link-arg=-Wl,-rpath,$ORIGIN";

          # WebRTC needs system SSL + crypto headers available
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.openssl
            pkgs.libva
            pkgs.libclang
            pkgs.libclang.devShells
            pkgs.libv4l.dev
            pkgs.linuxHeaders
          ];
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            pkg-config
            openssl
            livekit-libwebrtc
            libva
            libnice
            glib
            clang
            libxkbcommon
            wayland
            libclang
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            xorg.libXext
            xorg.libXinerama
            xorg.libXrender
            libv4l.dev
            libv4l
            linuxHeaders
            glibc.dev
          ];

          VULKAN_DIR = "${pkgs.vulkan-loader}";
          LIBV4L = "${pkgs.libv4l.dev}";
          WAYLAND_DIR= "${pkgs.wayland}";
          C_INCLUDE_PATH = "${pkgs.linuxHeaders}/include/:${pkgs.glibc.dev}/include/";
          LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib/:${pkgs.vulkan-loader}/lib/:${pkgs.libva.out}/lib/:${pkgs.libclang.lib}/lib/";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib/";
          RUST_BACKTRACE = "1";
        };
      });
}