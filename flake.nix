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
            livekit-libwebrtc
            libva
            wayland
            libxkbcommon
            pkgs.linuxHeaders
            libgcc
          ];

          # Some crates use system SSL paths or need environment hints
          RUSTFLAGS = "-C link-arg=-Wl,-rpath,$ORIGIN";

          # WebRTC needs system SSL + crypto headers available
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.openssl
            pkgs.libva
            pkgs.libclang
            pkgs.libv4l.dev
            pkgs.linuxHeaders
            pkgs.pipewire.dev

          ];
        };

        devShells.default = pkgs.mkShell {
          pure = true;

          buildInputs = with pkgs; [
            cargo
            pipewire.dev
            rustc
            pkg-config
            openssl
            livekit-libwebrtc
            libva
            libnice
            libxkbcommon
            wayland
            libclang
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools
            xorg.libX11
            libv4l.dev
            libv4l
            linuxHeaders
            libgcc
          ];
          shellHook = ''
            # unset PATH
            echo "PATH has been unset."
            # export PATH="${pkgs.libgcc.out}/bin:${pkgs.coreutils}/bin:${pkgs.rustup}/bin:${pkgs.llvmPackages.clangUseLLVM}/bin:${pkgs.pkg-config}/bin"
          '';
          CC = "gcc";
          VULKAN_DIR = "${pkgs.vulkan-loader}";
          LIBV4L = "${pkgs.libv4l.dev}";
          WAYLAND_DIR= "${pkgs.wayland}";
          C_INCLUDE_PATH = "${pkgs.linuxHeaders}/include/:${pkgs.libgcc.out}/include:${pkgs.glibc.dev}/include";
          CXX_INCLUDE_PATH = "${pkgs.libgcc.out}/include";
          CXXFLAGS = "-I${pkgs.libgcc.out}/include";
          CFLAGS = "-I${pkgs.libgcc.out}/include/c++/${pkgs.libgcc.version}/";
          LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib/:${pkgs.vulkan-loader}/lib/:${pkgs.libva.out}/lib/:${pkgs.libclang.lib}/lib/";
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib/";
          RUST_BACKTRACE = "1";
          PIPEWIRE = "${pkgs.pipewire.dev}";
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
            pkgs.openssl
            pkgs.libva
            pkgs.libclang
            pkgs.libv4l.dev
            pkgs.linuxHeaders
            pkgs.pipewire.dev

          ];
        };
      });
}