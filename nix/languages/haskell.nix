{ pkgs, ... }:
{
  packages = [
    # Cabal can use split without first downloading a Hackage package index.
    (pkgs.haskellPackages.ghcWithPackages (hs: [ hs.split ]))
    pkgs.cabal-install
    pkgs.haskellPackages.fourmolu
    # The default HLS wrapper supports the same haskellPackages compiler.
    pkgs.haskell-language-server
  ];
}
