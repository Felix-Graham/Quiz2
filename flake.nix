{
  description = "French Vocabulary Quiz - a minimalist, keyboard-only terminal quiz";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "frenchquiz";
          version = "0.1.0";
          src = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          # Ship the starter vocab folder alongside the binary so a fresh
          # `nix run` / `nix profile install` has something to quiz on.
          postInstall = ''
            mkdir -p $out/share/frenchquiz/vocab
            if [ -d vocab ]; then
              cp -r vocab/. $out/share/frenchquiz/vocab/ || true
            fi
          '';

          meta = with pkgs.lib; {
            description = "A minimalist, dark, keyboard-only terminal French vocabulary quiz";
            homepage = "https://github.com/Felix-Graham/French-Vocab-Quiz";
            license = licenses.mit;
            mainProgram = "frenchquiz";
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.cargo pkgs.rustc pkgs.rust-analyzer pkgs.clippy ];
        };
      });
}
