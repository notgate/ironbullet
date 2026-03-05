#!/usr/bin/env bash
# build-windows.sh — Cross-compile IronBullet for Windows (x86_64-pc-windows-llvmmingw)
#
# Scopes CMAKE_TOOLCHAIN_FILE, CMAKE_ASM_NASM_COMPILER, and BINDGEN_EXTRA_CLANG_ARGS
# to this invocation only. These vars CANNOT live in .cargo/config.toml [env] because
# that table is global and would break Linux native builds (boring-sys2 struct mismatch).

set -euo pipefail

# Resolve repo root (script lives in scripts/, one level down)
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LLVM_MINGW=/opt/llvm-mingw
SYSROOT="$LLVM_MINGW/x86_64-w64-mingw32"

# Pass the target JSON path directly (avoids needing RUST_TARGET_PATH).
# -Z build-std rebuilds Rust std with llvm-mingw (SEH ABI, not DWARF).
# -Z json-target-spec enables custom JSON target files.
# These -Z flags require nightly (+nightly).
CMAKE_ASM_NASM_COMPILER=/usr/bin/nasm \
CMAKE_TOOLCHAIN_FILE="$LLVM_MINGW/llvm-mingw-toolchain.cmake" \
BINDGEN_EXTRA_CLANG_ARGS="--target=x86_64-w64-windows-gnu --sysroot=$SYSROOT -I$SYSROOT/include -I/usr/lib/llvm-18/lib/clang/18/include" \
  /root/.cargo/bin/cargo +nightly build --release \
    --target "$REPO_ROOT/x86_64-pc-windows-llvmmingw.json" \
    -Z build-std=std,panic_abort \
    -Z json-target-spec \
    "$@"
