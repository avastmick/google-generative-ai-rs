{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
  let
    pversion = "v0.5.14";
    systemBuildInputs = system: pkgs: {
      ${system} = [ pkgs.iconv pkgs.openssl ]  ++ 
      (if pkgs.lib.strings.hasInfix "$system" "darwin"
      then [ pkgs.darwin.apple_sdk.frameworks.Security pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ]
      else [ ]);
      };
  in
  utils.lib.eachDefaultSystem( system:
  let
    pkgs = import nixpkgs { inherit system; };
    naersk-lib = pkgs.callPackage naersk { };

    src  =  pkgs.fetchzip {
      name = "src";
      url = "https://github.com/zurawiki/gptcommit/archive/refs/tags/${pversion}.tar.gz";
      hash = "sha256-xjaFr1y2Fd7IWbJlegnIsfS5/oMJYd6QTnwp7IK17xM=";
    };
  in
  {
    defaultPackage = naersk-lib.buildPackage  {
      inherit src;
      buildInputs = (systemBuildInputs system pkgs).${system};
    };

    devShell = with pkgs; mkShell {
      buildInputs = [ 
        cargo rustc rustfmt pre-commit rustPackages.clippy 
      ] ++ (systemBuildInputs system pkgs).${system};
      RUST_SRC_PATH = rustPlatform.rustLibSrc;
    };
  });
}
