{
  flake.overlays.default = final: _prev: {
    etilbudsavis-api = final.callPackage ./_package.nix {
      crate = "etilbudsavis-api";
    };
    etilbudsavis-cli = final.callPackage ./_package.nix {
      crate = "etilbudsavis-cli";
    };
    etilbudsavis-mcp = final.callPackage ./_package.nix {
      crate = "etilbudsavis-mcp";
    };
  };
}
