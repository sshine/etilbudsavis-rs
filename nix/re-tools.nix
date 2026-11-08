# APK acquisition, unpacking, decompilation, and binary inspection.
#
# Kept separate from the Rust shell so Phase 2 can run before any crate exists.
{ ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      devshells.default.packages = with pkgs; [
        # Acquisition (Google Play needs an AAS token; APKPure does not).
        apkeep

        # XAPK/APK unpacking and file census.
        unzip
        zip
        p7zip
        file
        tree
        findutils
        coreutils
        binwalk

        # Android resources, manifest, DEX, and signing.
        apktool
        jadx
        dex2jar
        android-tools
        aapt
        apksigner
        jdk

        # Native libraries and raw binaries.
        radare2
        binutils
        util-linux

        # Text and structured data.
        ripgrep
        jq
        xmlstarlet

        # Extraction and analysis scripts. No pip; use nixpkgs packages.
        python3
        python3Packages.requests
        python3Packages.lxml
      ];
    };
}
