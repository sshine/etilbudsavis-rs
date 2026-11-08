{
  inputs,
  ...
}:
{
  imports = [ inputs.devshell.flakeModule ];
  perSystem =
    { config, pkgs, ... }:
    {
      devshells.default = {
        packages = [
          config.treefmt.build.wrapper
          pkgs.git
          pkgs.just
          pkgs.curl
          pkgs.wget
          pkgs.openssl
          pkgs.pkg-config
          pkgs.jq
          pkgs.cargo-watch
        ];

        env = [
          {
            name = "RUST_BACKTRACE";
            value = "1";
          }
        ];

        devshell.motd = "";
      };
    };
}
