{ pkgs ? import <nixpkgs> {} }:
with pkgs; {
  illumium-api = stdenv.mkDerivation {
    name = "illumium-api";
    src = ".";
    buildInputs = [
      stdenv pkgconfig openssl libsodium llvmPackages.libclang #rustChannels.stable.rust
    ];
    LIBCLANG_PATH = "${llvmPackages.libclang}/lib";
  };
}
