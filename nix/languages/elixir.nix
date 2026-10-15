{ pkgs, ... }:
let
  beam = pkgs.beam.packages.erlang_27;
in {
  packages = [ pkgs.erlang_27 beam.elixir_1_18 beam.elixir-ls ];
}
