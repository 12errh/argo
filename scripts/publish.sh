#!/bin/bash
set -euo pipefail

# Argo Agent Framework - Release Script
# Usage: ./scripts/publish.sh <version>

VERSION=${1:?Usage: ./scripts/publish.sh <version>}
DRY_RUN=${DRY_RUN:-false}

echo "=========================================="
echo "Argo Agent Framework - Release v${VERSION}"
echo "=========================================="

# Validate version format
if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in semver format (e.g., 0.1.0)"
    exit 1
fi

# Run tests first
echo ""
echo "[1/8] Running tests..."
cargo test --all

# Format check
echo ""
echo "[2/8] Checking formatting..."
cargo fmt --all -- --check

# Clippy
echo ""
echo "[3/8] Running clippy..."
cargo clippy --all-targets --all-features

# Update version in Cargo.toml files
echo ""
echo "[4/8] Updating version to ${VERSION}..."
sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml
for crate in crates/*/; do
    if [ -f "${crate}Cargo.toml" ]; then
        sed -i "s/^version = \".*\"/version = \"${VERSION}\"/" "${crate}Cargo.toml"
    fi
done

# Commit version bump
echo ""
echo "[5/8] Committing version bump..."
git add -A
git commit -m "chore: release v${VERSION}"

# Tag
echo ""
echo "[6/8] Creating tag v${VERSION}..."
git tag -a "v${VERSION}" -m "Release v${VERSION}"

if [ "$DRY_RUN" = "false" ]; then
    # Push
    echo ""
    echo "[7/8] Pushing to origin..."
    git push origin main
    git push origin "v${VERSION}"

    # Publish crates
    echo ""
    echo "[8/8] Publishing to crates.io..."
    cargo publish -p argo-core
    sleep 30
    cargo publish -p argo-memory
    sleep 30
    cargo publish -p argo-heal
    sleep 30
    cargo publish -p argo-tools
    sleep 30
    cargo publish -p argo-mcp
    sleep 30
    cargo publish -p argo-observe
    sleep 30
    cargo publish -p argo-cli

    echo ""
    echo "=========================================="
    echo "Release v${VERSION} complete!"
    echo "=========================================="
    echo ""
    echo "Next steps:"
    echo "  1. GitHub Actions will build binaries and create release"
    echo "  2. Docker image will be published to GHCR"
    echo "  3. Run 'maturin upload' for PyPI (manual)"
    echo "  4. Run 'npm publish' for npm (manual)"
else
    echo ""
    echo "[7/8] DRY RUN - Skipping push and publish"
    echo "[8/8] DRY RUN - Skipping publish"
    echo ""
    echo "=========================================="
    echo "Dry run complete!"
    echo "=========================================="
fi
