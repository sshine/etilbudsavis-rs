# Default recipe: list available commands
default:
    @just --list

# Format Nix, Rust, Markdown, TOML, YAML, and shell
fmt:
    treefmt

# Check formatting without writing
fmt-check:
    treefmt --fail-on-change --no-cache

# Run the workspace tests
test:
    cargo test --workspace --all-features

# Run clippy over the workspace
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Build the workspace
build:
    cargo build --workspace --all-features

# Generate documentation
doc *args='':
    cargo doc --no-deps --workspace --all-features {{args}}

# Verify Cargo.lock is in step with the manifests
lock-check:
    cargo metadata --locked --format-version 1 > /dev/null

# Verify every member packages cleanly
package-check:
    cargo package --locked --workspace --allow-dirty --no-verify

# Run CI checks locally
ci: fmt-check lint test build doc lock-check package-check
    @echo "All CI checks passed!"

# Run the CLI
cli *args='':
    cargo run --quiet -p etilbudsavis-cli -- {{args}}

# Run the read-only CLI showcase against the live service
showcase:
    ./scripts/showcase.sh

# Run the MCP server over stdio
mcp *args='':
    cargo run --quiet -p etilbudsavis-mcp -- {{args}}

# Print the MCP tool schemas, which needs no credentials
mcp-tools *args='':
    cargo run --quiet -p etilbudsavis-mcp -- --list-tools {{args}}

# Run the live API tests against the real service, one at a time
live-test *args='':
    cargo test -p etilbudsavis-api --test live -- --ignored --nocapture --test-threads 1 {{args}}

# Print package, version, and SDK levels from the extracted base APK
apk-info:
    @aapt2 dump badging apk/extracted/com.eTilbudsavis.eTilbudsavis.apk | grep -E "^(package|minSdkVersion|targetSdkVersion|application-label)"

release_targets := "x86_64-unknown-linux-musl aarch64-unknown-linux-musl"

# Assert the release tag names the version cargo would publish
check-version version:
    @pkgid="$(cargo pkgid -p etilbudsavis-cli)"; crate="v${pkgid##*#}"; \
    if [ "$crate" != "{{version}}" ]; then \
        echo "tag {{version}} does not match crate version $crate" >&2; exit 1; \
    fi

# Build the static release artifacts and their checksums into dist/
dist version: (check-version version)
    rm -rf dist && mkdir -p dist
    for target in {{release_targets}}; do \
        cargo build --release --locked --target "$target" -p etilbudsavis-cli -p etilbudsavis-mcp; \
        name="etilbudsavis-{{version}}-$target"; \
        tar -czf "dist/$name.tar.gz" -C "target/$target/release" etilbudsavis etilbudsavis-mcp; \
        ( cd dist && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256" ); \
    done
    @ls -1 dist

# Print the SRI hashes nix/binary.nix pins, read from the published release
release-hashes version:
    @for target in {{release_targets}}; do \
        url="https://github.com/sshine/etilbudsavis-rs/releases/download/{{version}}/etilbudsavis-{{version}}-$target.tar.gz"; \
        hash="$(nix store prefetch-file --json --hash-type sha256 "$url" | jq -r .hash)"; \
        printf '"%s" = "%s";\n' "${target%%-*}-linux" "$hash"; \
    done
