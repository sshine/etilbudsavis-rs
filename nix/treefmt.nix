{
  inputs,
  ...
}:
{
  imports = [ inputs.treefmt-nix.flakeModule ];
  perSystem =
    { pkgs, ... }:
    let
      rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
    in
    {
      treefmt = {
        projectRootFile = "flake.nix";

        settings.global.excludes = [
          "AGENTS.md"
          "apk/**"
          "secrets/**"
          "re/decompiled/**"
          "re/extracted/**"
          "re/analysis/*.pretty.js"
          "re/analysis/*.ndjson"
          "re/reports/**"
          ".direnv/**"
          "target/**"
          "result"
        ];

        programs.nixfmt.enable = true;
        programs.rustfmt = {
          enable = true;
          package = rust-toolchain;
        };
        programs.mdformat = {
          enable = true;
          plugins = ps: [ ps.mdformat-frontmatter ];
          settings.number = true;
        };
        programs.taplo.enable = true;
        programs.yamlfmt = {
          enable = true;
          settings.formatter.retain_line_breaks_single = true;
        };
        programs.shfmt.enable = true;
      };
    };
}
