{
	inputs = {
		nixpkgs.url     = "github:nixos/nixpkgs/nixpkgs-unstable";
		flake-utils.url = "github:numtide/flake-utils";
		rust-overlay = {
			url = "github:oxalica/rust-overlay";
			inputs.nixpkgs.follows = "nixpkgs";
		};
		smallstr = {
			url = "github:SKyletoft/smallstr";
			flake = false;
		};
		smallvec = {
			url = "github:SKyletoft/rust-smallvec/v1";
			flake = false;
		};
	};

	outputs = { self, nixpkgs, rust-overlay, flake-utils, smallstr, smallvec }:
		flake-utils.lib.eachDefaultSystem(system:
			let
				pkgs = import nixpkgs {
					inherit system;
					overlays = [ rust-overlay.overlays.default ];
				};

				rustToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
					extensions = [ "rust-src" "rust-analyzer" "miri" ];
					targets = [ "x86_64-unknown-linux-gnu" ];
				});

				rustPlatform = pkgs.makeRustPlatform {
					cargo = rustToolchain;
					rustc = rustToolchain;
				};

				devBuildInputs = with pkgs; [
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

					rlwrap
					custom-agda
					ghc
				];

				alisp-unwrapped = rustPlatform.buildRustPackage {
					pname = "alisp";
					version = "0.0.1";
					src = pkgs.runCommand "alisp-source" {} ''
						cp -r ${self}/. $out/
						chmod -R u+w $out
						rm -rf $out/smallstr $out/smallvec
						cp -r ${smallstr} $out/smallstr
						cp -r ${smallvec} $out/smallvec
						chmod -R u+w $out/smallstr $out/smallvec
					'';
					cargoLock.lockFile = ./Cargo.lock;
					doCheck = false;
				};

				custom-agda = pkgs.agda.withPackages (p: with p; [
					standard-library
				]);

				alisp = pkgs.runCommand "alisp-0.0.1" {
					nativeBuildInputs = [ pkgs.makeWrapper ];
				}
				''
					mkdir -p $out/bin
					makeWrapper ${pkgs.rlwrap}/bin/rlwrap $out/bin/alisp \
						--add-flags ${alisp-unwrapped}/bin/repl
				'';
			in {
				devShells.default = pkgs.mkShell {
					packages = devBuildInputs;
				};
				packages.alisp-unwrapped = alisp-unwrapped;
				packages.alisp = alisp;
				packages.default = alisp;
			}
		);
}
