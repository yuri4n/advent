{ pkgs, ... }:
{
  packages = [
    # Cabal can use the parser and solution dependencies without a Hackage index.
    (pkgs.haskellPackages.ghcWithPackages (hs: [ hs.split hs.megaparsec ]))
    pkgs.cabal-install
    pkgs.haskellPackages.fourmolu
    # The default HLS wrapper supports the same haskellPackages compiler.
    pkgs.haskell-language-server
  ];
}
