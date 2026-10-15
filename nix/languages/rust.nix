{ rust, ... }:
{
  packages = [ rust ];
  environment.RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";
}
