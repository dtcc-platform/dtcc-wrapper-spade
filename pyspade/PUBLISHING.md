# Publishing pyspade to PyPI

This guide explains how to build and publish pyspade wheels to PyPI.

## Prerequisites

1. **PyPI Account**: Create accounts on:
   - [PyPI](https://pypi.org/account/register/) (production)
   - [TestPyPI](https://test.pypi.org/account/register/) (testing)

2. **API Token**: Generate API tokens:
   - Go to Account Settings → API tokens
   - Create token for "Entire account" or specific project
   - Save the token securely

3. **GitHub Secrets**: Add to repository secrets:
   - `PYPI_API_TOKEN`: Your PyPI API token
   - `TEST_PYPI_API_TOKEN`: Your TestPyPI API token (optional)

## Local Build and Testing

### Build wheels locally

```bash
# Install maturin
pip install maturin

# Build for current platform
cd pyspade
maturin build --release

# Output: target/wheels/pyspade-*.whl
```

### Test the wheel

```bash
# Install locally
pip install target/wheels/pyspade-*.whl

# Run tests
python examples/basic.py
```

### Build for all Python versions

```bash
# Build for Python 3.10, 3.11, 3.12
maturin build --release --interpreter python3.10
maturin build --release --interpreter python3.11
maturin build --release --interpreter python3.12
```

## Publishing with GitHub Actions

### Automatic builds on push

The `.github/workflows/wheels.yml` workflow automatically:
1. Builds wheels for Linux (x86_64, aarch64)
2. Builds wheels for macOS (x86_64, arm64)
3. Builds wheels for Windows (x64)
4. Builds source distribution (sdist)
5. Uploads artifacts

### Publishing a release

1. **Update version** in `Cargo.toml` and `pyproject.toml`:
   ```toml
   version = "0.1.0"
   ```

2. **Commit changes**:
   ```bash
   git add .
   git commit -m "Bump version to 0.1.0"
   git push
   ```

3. **Create and push tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

4. **Automatic publishing**: GitHub Actions will:
   - Build all wheels
   - Publish to PyPI automatically if tag starts with `v`

## Manual Publishing

### Publish to TestPyPI first

```bash
# Build wheels
maturin build --release

# Publish to TestPyPI
maturin upload --repository testpypi target/wheels/*
# Or with token:
MATURIN_PYPI_TOKEN=<your-test-token> maturin upload --repository testpypi target/wheels/*
```

### Test installation from TestPyPI

```bash
pip install --index-url https://test.pypi.org/simple/ pyspade
```

### Publish to PyPI

```bash
# Build wheels
maturin build --release

# Publish to PyPI
maturin upload target/wheels/*
# Or with token:
MATURIN_PYPI_TOKEN=<your-token> maturin upload target/wheels/*
```

## Cross-platform Builds

### Using maturin with Docker (Linux)

```bash
# Build manylinux wheels
docker run --rm -v $(pwd):/io \
  ghcr.io/pyo3/maturin build --release \
  --manifest-path /io/pyspade/Cargo.toml
```

### Using cross-compilation (macOS)

```bash
# Install targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for both architectures
maturin build --release --target x86_64-apple-darwin
maturin build --release --target aarch64-apple-darwin

# Or build universal2 wheel
maturin build --release --target universal2-apple-darwin
```

### Using cross (Linux ARM)

```bash
# Install cross
cargo install cross

# Build for aarch64
cross build --release --target aarch64-unknown-linux-gnu
```

## Verifying Wheels

### Check wheel contents

```bash
unzip -l target/wheels/pyspade-*.whl
```

### Check metadata

```bash
pip install pkginfo
pkginfo target/wheels/pyspade-*.whl
```

### Test on different platforms

Use [cibuildwheel](https://cibuildwheel.readthedocs.io/) for comprehensive testing:

```bash
pip install cibuildwheel
cibuildwheel --platform linux
```

## Troubleshooting

### Wheel not found for platform

- Check `pyproject.toml` classifiers
- Verify platform-specific builds succeeded
- Check maturin target specification

### Import errors

- Verify Python version compatibility
- Check dynamic library dependencies: `ldd` (Linux), `otool -L` (macOS)
- Ensure proper manylinux tags

### Version conflicts

- Bump version in both `Cargo.toml` and `pyproject.toml`
- Use `--skip-existing` to avoid re-uploading same version
- Delete old builds: `rm -rf target/wheels/*`

## Release Checklist

Before releasing:

- [ ] Update version numbers
- [ ] Update CHANGELOG.md
- [ ] Run all tests locally
- [ ] Build and test wheels for all platforms
- [ ] Test installation from TestPyPI
- [ ] Review README.md examples
- [ ] Check all links in documentation
- [ ] Verify license files present
- [ ] Tag release and push
- [ ] Monitor GitHub Actions workflow
- [ ] Verify PyPI upload
- [ ] Test `pip install pyspade` from PyPI

## Resources

- [Maturin Documentation](https://www.maturin.rs/)
- [PyO3 Guide](https://pyo3.rs/)
- [Python Packaging Guide](https://packaging.python.org/)
- [PyPI Upload Guide](https://packaging.python.org/tutorials/packaging-projects/)
