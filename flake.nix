{
  description = "Rust Bevy development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        toolchain = fenix.packages.${system}.stable.withComponents [
          "cargo"
          "rust-src"
          "rustc"
          "rustfmt"
          "rust-analyzer"
        ];
        pkgs = nixpkgs.legacyPackages.${system};
        platform = pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        };
        buildInputs = with pkgs; [
          # macOS specific dependencies
          darwin.apple_sdk.frameworks.CoreServices
          darwin.apple_sdk.frameworks.CoreGraphics
          darwin.apple_sdk.frameworks.Foundation
          darwin.apple_sdk.frameworks.Metal
          darwin.apple_sdk.frameworks.QuartzCore
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
          darwin.apple_sdk.frameworks.AppKit
          darwin.apple_sdk.frameworks.AudioUnit
          darwin.apple_sdk.frameworks.CoreAudio
          darwin.apple_sdk.frameworks.CoreHaptics
          darwin.apple_sdk.frameworks.CoreMedia
          darwin.apple_sdk.frameworks.GameController
          darwin.apple_sdk.frameworks.OpenAL
          darwin.libobjc
        ];

        nativeBuildInputs = [
          toolchain
          platform.bindgenHook
        ] ++ (with pkgs; [
          bacon
        ]);
      in
      {
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;
        };
      }
    );
}
