{ pkgs, ... }:
{
  packages = [ pkgs.scala-cli pkgs.jdk21 pkgs.metals ];
  environment = {
    JAVA_HOME = "${pkgs.jdk21}";
    AOC_JAVA = "${pkgs.jdk21}/bin/java";
    SCALA_CLI = "${pkgs.scala-cli}/bin/scala-cli";
  };
}
