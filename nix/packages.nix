{
  perSystem =
    { pkgs, lib, ... }:
    rec {
      checks.etilbudsavis-api = packages.etilbudsavis-api;
      checks.etilbudsavis-cli = packages.etilbudsavis-cli;
      checks.etilbudsavis-mcp = packages.etilbudsavis-mcp;

      packages.etilbudsavis-api = pkgs.callPackage ./_package.nix {
        crate = "etilbudsavis-api";
      };
      packages.etilbudsavis-cli = pkgs.callPackage ./_package.nix {
        crate = "etilbudsavis-cli";
      };
      packages.etilbudsavis-mcp = pkgs.callPackage ./_package.nix {
        crate = "etilbudsavis-mcp";
      };

      packages.default = packages.etilbudsavis-cli;

      apps.etilbudsavis-cli = {
        type = "app";
        program = lib.getExe packages.etilbudsavis-cli;
      };
      apps.etilbudsavis-mcp = {
        type = "app";
        program = lib.getExe packages.etilbudsavis-mcp;
      };
      apps.default = apps.etilbudsavis-cli;
    };
}
