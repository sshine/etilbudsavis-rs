# Cross-compilation for the release artifacts.
#
# The release publishes static musl binaries for Linux. rustc finds x86_64
# musl's crt objects in its own self-contained directory; a foreign architecture
# needs a cross linker.
{ ... }:
{
  perSystem =
    { pkgs, lib, ... }:
    let
      aarch64-cc = pkgs.pkgsCross.aarch64-multiplatform-musl.stdenv.cc;
      aarch64-linker = "${aarch64-cc}/bin/aarch64-unknown-linux-musl-cc";
      x86_64-cc = pkgs.pkgsCross.musl64.stdenv.cc;
      x86_64-linker = "${x86_64-cc}/bin/x86_64-unknown-linux-musl-cc";
    in
    {
      devshells.default = {
        packages = [
          aarch64-cc
          x86_64-cc
        ];

        env = [
          {
            name = "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER";
            value = aarch64-linker;
          }
          {
            name = "CC_aarch64_unknown_linux_musl";
            value = aarch64-linker;
          }
          {
            name = "AR_aarch64_unknown_linux_musl";
            value = "${aarch64-cc}/bin/aarch64-unknown-linux-musl-ar";
          }
          {
            name = "CC_x86_64_unknown_linux_musl";
            value = x86_64-linker;
          }
          {
            name = "AR_x86_64_unknown_linux_musl";
            value = "${x86_64-cc}/bin/x86_64-unknown-linux-musl-ar";
          }
        ];
      };

      packages.aarch64-musl-cc = aarch64-cc;

      apps.release-linker = {
        type = "app";
        program = lib.getExe (
          pkgs.writeShellApplication {
            name = "release-linker";
            text = ''echo "${aarch64-linker}"'';
          }
        );
      };
    };
}
