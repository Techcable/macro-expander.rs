set working-directory := "."

test: _check && format-check
    # Running rust tests
    cargo nextest run --all --all-features

check: _check && format-check

_check: _clippy
    # Checking documentation
    cargo doc --quiet --no-deps --workspace

_clippy:
    # Checking for compilation errors
    cargo clippy --all --all-features

format-check:
    # Checking formatting
    cargo fmt --check

format:
    # Formatting Rust code
    cargo fmt


