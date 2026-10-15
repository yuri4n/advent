{
  description = "Advent of Code: five language toolchains and the aoc runner";

  inputs = {
    nixpkgs.url = "git+https://github.com/NixOS/nixpkgs.git?ref=nixos-25.11&shallow=1";
    rust-overlay = {
      url = "git+https://github.com/oxalica/rust-overlay.git?ref=master&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSystem = nixpkgs.lib.genAttrs systems;
      environments = forEachSystem (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          # The shell and packaged CLI use the same pin as non-Nix developers.
          rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
          aoc = rustPlatform.buildRustPackage {
            pname = "advent-toolkit";
            version = "0.1.0";
            # Adapter integration tests use paths relative to toolkit/.
            src = pkgs.lib.fileset.toSource {
              root = ./.;
              fileset = pkgs.lib.fileset.unions [ ./toolkit ./scripts/languages ];
            };
            cargoRoot = "toolkit";
            buildAndTestSubdir = "toolkit";
            cargoLock.lockFile = ./toolkit/Cargo.lock;
            nativeBuildInputs = [ pkgs.makeWrapper ];
            nativeCheckInputs = [ pkgs.bash pkgs.coreutils ];
            # Integration tests create scripts; Nix sandboxes have no /bin/sh.
            postPatch = ''
              substituteInPlace toolkit/tests/*.rs \
                --replace-quiet '#!/bin/sh' '#!${pkgs.runtimeShell}'
            '';
            postInstall = ''
              wrapProgram "$out/bin/aoc" \
                --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.bash pkgs.coreutils pkgs.curl ]}
            '';
            meta = {
              description = "Advent of Code workspace runner";
              mainProgram = "aoc";
              platforms = systems;
            };
          };
          # Adding nix/languages/<name>.nix also adds a named development shell.
          languageFiles = pkgs.lib.filterAttrs
            (name: type: type == "regular" && pkgs.lib.hasSuffix ".nix" name)
            (builtins.readDir ./nix/languages);
          languages = assert pkgs.lib.assertMsg
            (!(builtins.any (name: builtins.hasAttr "${name}.nix" languageFiles) [ "base" "default" "all" ]))
            "nix/languages: base, default, and all are reserved shell names";
            pkgs.lib.mapAttrs'
            (file: _: pkgs.lib.nameValuePair (pkgs.lib.removeSuffix ".nix" file)
              (import (./nix/languages + "/${file}") { inherit pkgs rust; }))
            languageFiles;
        in {
          inherit aoc;
          formatter = pkgs.nixfmt-rfc-style;
          shells =
            let
              commonPackages = [
                aoc
                pkgs.bashInteractive
                pkgs.coreutils
                pkgs.findutils
                pkgs.gnused
                pkgs.gnugrep
                pkgs.git
                pkgs.curl
                pkgs.python3
                pkgs.jq
                pkgs.ripgrep
                pkgs.fd
                pkgs.fzf
                pkgs.neovim
              ];
              shell = name: packages: environment: pkgs.mkShell (environment // {
                packages = commonPackages ++ packages;
                AOC_NIX_LANGUAGE = name;
              });
            in
              (builtins.mapAttrs (name: language:
                shell name language.packages (language.environment or {})
              ) languages) // {
                default = shell "base" [] {};
                all = shell "all"
                  (pkgs.lib.concatMap (language: language.packages) (builtins.attrValues languages))
                  (builtins.foldl' (all: language: all // (language.environment or {})) {} (builtins.attrValues languages));
              };
        });
    in {
      packages = forEachSystem (system: {
        default = environments.${system}.aoc;
        aoc = environments.${system}.aoc;
      });
      apps = forEachSystem (system: {
        default = {
          type = "app";
          program = "${environments.${system}.aoc}/bin/aoc";
          meta.description = "Advent of Code workspace runner";
        };
        aoc = {
          type = "app";
          program = "${environments.${system}.aoc}/bin/aoc";
          meta.description = "Advent of Code workspace runner";
        };
      });
      devShells = forEachSystem (system: environments.${system}.shells);
      checks = forEachSystem (system: {
        aoc = environments.${system}.aoc;
      });
      formatter = forEachSystem (system: environments.${system}.formatter);
    };
}
