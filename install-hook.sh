#!/usr/bin/env bash
# Install pre-commit hooks for slidet
# Requires: pre-commit (pip/brew), cargo, rustfmt, clippy
set -euo pipefail

echo "Installing pre-commit hooks..."
pre-commit install
echo ""
echo "Running all hooks against all files to verify..."
pre-commit run --all-files
echo ""
echo "Done. Hooks will run on every 'git commit'."
echo "To skip hooks temporarily: git commit --no-verify"
