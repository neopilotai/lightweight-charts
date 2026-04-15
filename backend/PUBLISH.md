# Publishing to crates.io

## 1. Login to crates.io

```bash
cargo login
```

Enter your crates.io API token when prompted.

## 2. Publish

```bash
cd backend
cargo publish
```

## 3. For version updates

Bump version in `Cargo.toml`, then:

```bash
cargo publish --dry-run  # Test first
cargo publish             # Publish
```

## Notes

- Package name on crates.io: `lightweight-charts-backend`
- Run `cargo login` first with your crates.io credentials
- First publish requires verification on crates.io