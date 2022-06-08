{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
        rec {
          # `nix build`
          packages.ord = naersk-lib.buildPackage {
            pname = "ord";
            root = ./.;
            nativeBuildInputs = with pkgs; [ rustc cargo openssl pkg-config ];
          };
          defaultPackage = packages.ord;

          # `nix run`
          apps.ord = flake-utils.lib.mkApp {
            drv = packages.ord;
          };
          defaultApp = apps.ord;

          # `nix develop`
          devShell = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ rustc cargo openssl pkg-config ];
          };
        }
    );
}
