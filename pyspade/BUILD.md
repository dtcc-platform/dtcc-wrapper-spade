# Building pyspade

## How Spade is Fetched

pyspade uses a **Git dependency** in `Cargo.toml` to automatically fetch the Spade library during build:

```toml
[dependencies]
spade = { git = "https://github.com/Stoeoef/spade", tag = "v2.15.0" }
```

This means:
- ✅ **No submodules needed** - Cargo fetches Spade automatically from GitHub
- ✅ **Works for PyPI** - Source distributions (sdist) build correctly
- ✅ **CI/CD friendly** - GitHub Actions doesn't need `submodules: recursive`
- ✅ **Version locked** - Always builds with Spade v2.15.0

## Local Development Build

### Prerequisites

- Rust toolchain (1.70+): https://rustup.rs
- Python 3.10+
- Maturin: `pip install maturin`

### Build Steps

```bash
cd pyspade

# Build wheel
python3 -m maturin build --release --interpreter python3.11

# Or build and install in development mode
python3 -m maturin develop --release
```

### First Build

On the first build, you'll see:

```
Updating git repository `https://github.com/Stoeoef/spade`
Locking 1 package to latest compatible version
Adding spade v2.15.0 (https://github.com/Stoeoef/spade?tag=v2.15.0#b38f333e)
```

This is normal! Cargo is fetching Spade from GitHub. Subsequent builds reuse the cached version.

## Multi-Platform Builds

### Linux (manylinux)

```bash
docker run --rm -v $(pwd):/io \
  ghcr.io/pyo3/maturin build --release \
  --manifest-path /io/Cargo.toml
```

### macOS Universal

```bash
# Build for both Intel and Apple Silicon
rustup target add x86_64-apple-darwin aarch64-apple-darwin

maturin build --release --target x86_64-apple-darwin
maturin build --release --target aarch64-apple-darwin

# Or build universal2 wheel (both architectures in one wheel)
maturin build --release --target universal2-apple-darwin
```

### Windows

```bash
# Build for x64
maturin build --release --target x86_64-pc-windows-msvc
```

## GitHub Actions

The `.github/workflows/wheels.yml` automatically:

1. Checks out the repository (no submodule needed!)
2. Cargo fetches Spade from Git during build
3. Builds wheels for all platforms
4. Publishes to PyPI on tagged releases

## Updating Spade Version

To update to a newer version of Spade:

1. Check available tags: https://github.com/Stoeoef/spade/tags

2. Update `Cargo.toml`:
   ```toml
   [dependencies]
   spade = { git = "https://github.com/Stoeoef/spade", tag = "v2.16.0" }
   ```

3. Update lock file:
   ```bash
   cargo update spade
   ```

4. Test build:
   ```bash
   maturin build --release
   ```

5. Commit changes:
   ```bash
   git add Cargo.toml Cargo.lock
   git commit -m "Update Spade to v2.16.0"
   ```

## Offline Builds

If you need to build offline (after first fetch):

```bash
# Cargo caches Git dependencies in:
~/.cargo/git/checkouts/spade-*

# Build offline (uses cache)
cargo build --offline
```

## Troubleshooting

### "Couldn't detect the binding type"

**Problem:** Maturin can't find `pyproject.toml`

**Solution:** Run from the `pyspade/` directory:
```bash
cd pyspade
maturin build --release
```

### "failed to load source for dependency `spade`"

**Problem:** Network issue or Git not installed

**Solution:**
1. Check internet connection
2. Verify Git is installed: `git --version`
3. Try manual clone:
   ```bash
   git clone https://github.com/Stoeoef/spade
   ```

### "linker errors" on macOS

**Problem:** Rust not configured for Python linking

**Solution:** Let maturin handle it:
```bash
maturin build --release --interpreter $(which python3.11)
```

### Build is slow

**First build:** ~10-15 seconds (downloads Spade + compiles)
**Subsequent builds:** ~5-8 seconds (uses cache)

To speed up:
```bash
# Use sccache (shared compilation cache)
cargo install sccache
export RUSTC_WRAPPER=sccache

# Or use mold linker (Linux)
maturin build --release -- -C link-arg=-fuse-ld=mold
```

## Verifying the Build

```bash
# Check wheel contents
unzip -l target/wheels/pyspade-*.whl

# Test import
pip install target/wheels/pyspade-*.whl
python -c "import pyspade; print(pyspade.triangulate.__doc__)"

# Run examples
python examples/basic.py
```

## Build Artifacts

```
pyspade/
├── target/
│   ├── wheels/               # Built wheels
│   │   └── pyspade-*.whl
│   ├── release/              # Compiled binaries
│   └── CACHEDIR.TAG
│
└── Cargo.lock                # Dependency lock file (commit this!)
```

**Important:** Always commit `Cargo.lock` to ensure reproducible builds!

## Clean Build

```bash
# Remove all build artifacts
cargo clean

# Remove only wheels
rm -rf target/wheels/*

# Full clean (including Cargo cache)
rm -rf target/
```
