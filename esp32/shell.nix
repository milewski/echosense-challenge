with import <nixpkgs> {};
mkShell {
    nativeBuildInputs = with pkgs; [
        python3
        (python3.withPackages (ps: with ps; [
          pip
          setuptools
          virtualenv
        ]))
#        cargo
#        rustup
#        espup
#        espflash
#        ldproxy
#        libclang
#        zlib
#        (rustPlatform.buildRustPackage {
#            pname = "espup";
#            version = "0.13.0";
#            src = fetchCrate {
#                pname = "espup";
#                version = "0.13.0";
#                sha256 = "sha256-bGVvxcmga6S4TlJCSZiyWQJvd3U3uMjWxgrNU2n6FHk=";
#            };
#            cargoHash = "sha256-n2tLG3logtc73Ut/R1UGuLSm7MpZ4Bxp/08SOhGL+80=";
#        })
    ];
#      shellHook = ''
#        export LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
#        export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib";
#      '';
}