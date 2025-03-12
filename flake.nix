{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";

    fenix-pkg.url = "github:nix-community/fenix";
    fenix-pkg.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {nixpkgs, fenix-pkg, ...}: let
    system = "x86_64-linux";
    pkgs = import nixpkgs { inherit system; };
    lib = pkgs.lib;
    fenix = fenix-pkg.packages.${system};

    toolchain = fenix.toolchainOf {
      channel = "stable";
      date = "2025-02-20";
      sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
    };

    backendBuildInputs = [
      (toolchain.withComponents ["rustc" "cargo" "rust-src"])
    ];

    frontendBuildInputs = with pkgs; [
      nodejs_22
      pnpm_10
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = backendBuildInputs ++ frontendBuildInputs;

      shellHook = ''
        cd frontend && pnpm install --frozen-lockfile || pnpm install
      '';
    };
  };
}
