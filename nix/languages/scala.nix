{ pkgs, ... }:
let
  # These packages otherwise wrap their executables with Java 21, overriding
  # JAVA_HOME. RC7's published macro classes require Java 25 in both tools.
  jdk = pkgs.jdk25;
  scalaCli = pkgs.scala-cli.override { jre = jdk; };
  metals = pkgs.metals.override { jre = jdk; };
in {
  packages = [ scalaCli jdk metals ];
  environment = {
    JAVA_HOME = jdk.home;
    AOC_JAVA = "${jdk}/bin/java";
    SCALA_CLI = "${scalaCli}/bin/scala-cli";
  };
}
