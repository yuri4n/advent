{ pkgs, ... }:
{
  packages = [ pkgs.scala-cli pkgs.jdk25 pkgs.metals ];
  environment = {
    JAVA_HOME = "${pkgs.jdk25}";
    AOC_JAVA = "${pkgs.jdk25}/bin/java";
    SCALA_CLI = "${pkgs.scala-cli}/bin/scala-cli";
  };
}
