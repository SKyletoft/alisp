{
	inputs = {
		nixpkgs.url     = "github:nixos/nixpkgs/nixpkgs-unstable";
		flake-utils.url = "github:numtide/flake-utils";
		rust-overlay = {
			url = "github:oxalica/rust-overlay";
			inputs.nixpkgs.follows = "nixpkgs";
		};
	};

	outputs = { self, nixpkgs, rust-overlay, flake-utils }:
		flake-utils.lib.eachDefaultSystem(system:
			let
				pkgs = import nixpkgs {
					inherit system;
					overlays = [( import rust-overlay )];
				};
				rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
					extensions = [ "rust-src" "rust-analyzer" "miri" ];
					targets = [ "x86_64-unknown-linux-gnu" ];
				});
				nativeBuildInputs = with pkgs; [
					rustToolchain
					cargo-expand
					cargo-show-asm
					cargo-flamegraph
					cargo-fuzz

					llvmPackages_22.clang-tools
					valgrind
					perf
					gdb
					gf

					kdePackages.kcachegrind
				];
			in {
				devShells.default = pkgs.mkShell {
					packages = nativeBuildInputs;
				};
				packages.default = pkgs.rustPlatform.buildRustPackage {
					pname = "alisp";
					version = "0.0.1";
					src = self;
					cargoLock.lockFile = ./Cargo.lock;

					inherit nativeBuildInputs;
				};
			}
		);
}
