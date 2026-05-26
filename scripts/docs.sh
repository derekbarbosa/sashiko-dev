#!/usr/bin/env bash
# Build the GitHub Pages site.
# Mirrors: .github/workflows/docs.yml -> build
set -euo pipefail

echo "==> Build rustdoc"
RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo doc --no-deps --document-private-items

echo "==> Assemble Pages site"
mkdir -p target/pages/rustdoc
cp -r pages/* target/pages/
cp static/favicon.ico target/pages/
cp static/logo.png target/pages/
cp -r target/doc/* target/pages/rustdoc/

echo "==> Pages site assembled in target/pages/"
echo "    Serve with: python3 -m http.server 8787 --directory target/pages"
