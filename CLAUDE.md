# CLAUDE.md

See @README.md for project details. Always update README.md when project details change.

## Documentation

- Find dependency docs in target/doc-md/index.md.
- Regenerate after adding a dependency with `cargo doc-md`.

## Rules

- Use `cargo add` to add dependencies to get latest versions.
- Always use `cargo clippy` and fix all issues before committing
- Run `cargo fmt` before committing (after clippy passes)
- Edition 2024 is correct

## Releasing

Don't publish or release without asking me.

1. Bump version in Cargo.toml (patch version unless told otherwise)
2. Commit: `git commit -am "Release vX.Y.Z"`
3. Publish to crates.io: `cargo publish`
4. Tag and push: `git tag vX.Y.Z && git push && git push --tags`
5. GitHub Actions builds binaries and updates Homebrew tap automatically
