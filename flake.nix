{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";

    fenix-pkg.url = "github:nix-community/fenix";
    fenix-pkg.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    nixpkgs,
    fenix-pkg,
    ...
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {inherit system;};
    lib = pkgs.lib;
    fenix = fenix-pkg.packages.${system};

    toolchain = fenix.toolchainOf {
      channel = "stable";
      date = "2025-02-20";
      sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
    };

    commonBuildInputs = with pkgs; [
      just
    ];

    backendBuildInputs = with pkgs; [
      (toolchain.withComponents ["rustc" "cargo" "rust-src"])

      # LXC vendor
      ouch
      rsync
    ];

    frontendBuildInputs = with pkgs; [
      nodejs_22
      pnpm_10
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs =
        commonBuildInputs
        ++ backendBuildInputs
        ++ frontendBuildInputs;

      LIBSECCOMP_LIB_PATH = "${lib.makeLibraryPath [pkgs.libseccomp]}";
      LD_LIBRARY_PATH = "${lib.makeLibraryPath [pkgs.libseccomp]}";

      shellHook = ''
        cd frontend && pnpm install --frozen-lockfile || pnpm install
        just setup-vendor-if-not
      '';
    };
  };
}
