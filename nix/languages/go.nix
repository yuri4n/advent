{ pkgs, ... }:
{
  packages = [ pkgs.go pkgs.gopls ];
  # Do not silently replace the compiler selected by flake.lock.
  environment.GOTOOLCHAIN = "local";
}
