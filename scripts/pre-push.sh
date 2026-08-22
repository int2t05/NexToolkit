#!/usr/bin/env bash
# Pre-push gate: mirrors CI core-cli + frontend jobs.
# Run manually before push, or install as a git hook:
#   cp scripts/pre-push.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push
# Fails fast on the first broken gate. Skipped if not on a rust/node project.
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail() { echo "❌ pre-push: $*" >&2; exit 1; }
ok()   { echo "✓ $*"; }

# 1. format check (CI: cargo fmt --check --all)
if [ -f Cargo.toml ]; then
  cargo fmt --check --all || fail "cargo fmt --check failed — run 'cargo fmt --all'"
  ok "cargo fmt"
fi

# 2. clippy (CI scope: core + cli; gui needs dist/, verified in CI tauri-build)
if [ -f Cargo.toml ]; then
  cargo clippy -p nextool-core -p nextool-cli --all-targets -- -D warnings \
    || fail "clippy failed (core+cli, -D warnings)"
  ok "clippy (core+cli)"
fi

# 3. test (CI: real data, no mock)
if [ -f Cargo.toml ]; then
  cargo test -p nextool-core -p nextool-cli || fail "cargo test failed"
  ok "cargo test (core+cli)"
fi

# 4. workspace build (catches breakage in crates CI doesn't lint, e.g. gui needs dist)
if [ -f Cargo.toml ] && [ -d dist ]; then
  cargo build --workspace || fail "cargo build --workspace failed (did you run 'npm run build' for gui?)"
  ok "cargo build --workspace"
elif [ -f Cargo.toml ]; then
  # dist missing → gui can't build; build non-gui crates only
  cargo build -p nextool-core -p nextool-cli || fail "cargo build failed"
  ok "cargo build (core+cli; dist/ missing — gui build skipped, run 'npm run build')"
fi

# 5. frontend type check + build (CI frontend job)
#    svelte-check 限定 workspace=src:避免本地扫到 git-ignored reference/ 竞品源码的 svelte.config
#    (CI 干净 checkout 无 reference/,`npm run check` 即可)
if [ -f package.json ]; then
  [ -d node_modules ] || npm ci
  ROOT="$(pwd)"
  npx svelte-check --tsconfig "$ROOT/tsconfig.json" --workspace "$ROOT/src" --threshold error \
    || fail "svelte-check failed"
  ok "svelte-check"
  npm run build || fail "vite build failed (dist/ regenerated)"
  ok "npm run build"
fi

echo "✅ pre-push: all gates passed"
