all: build test xtests
all-release: build-release test-release xtests-release

export HEXIT_DEBUG := ""


# compiles the hexit binary
@build:
    cargo build

# compiles the hexit binary (in release mode)
@build-release:
    cargo build --release --verbose


# runs unit tests
@test:
    cargo test --workspace -- --quiet

# runs unit tests (in release mode)
@test-release:
    cargo test --release --workspace --verbose

# runs mutation tests
@test-mutation:
    cargo +nightly test    --package hexit-lang --features=hexit-lang/with_mutagen -- --quiet
    cargo +nightly mutagen --package hexit-lang --features=hexit-lang/with_mutagen


# runs extended integration tests
@xtests:
    specsheet xtests/*.toml -O cmd.target.hexit="${CARGO_TARGET_DIR:-../target}/debug/hexit" -shide

# runs extended integration tests (in release mode)
@xtests-release:
    specsheet xtests/*.toml -O cmd.target.hexit="${CARGO_TARGET_DIR:-../target}/release/hexit"


# lints the code
@clippy:
    touch hexit-lang/src/lib.rs
    cargo clippy

# generates a code coverage report using tarpaulin via docker
@coverage-docker:
    docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin cargo tarpaulin --workspace --out Html

# updates dependency versions, and checks for outdated ones
@update-deps:
    cargo update
    command -v cargo-outdated >/dev/null || (echo "cargo-outdated not installed" && exit 1)
    cargo outdated

# lists unused dependencies
@unused-deps:
    command -v cargo-udeps >/dev/null || (echo "cargo-udeps not installed" && exit 1)
    cargo +nightly udeps

# prints versions of the necessary build tools
@versions:
    rustc --version
    cargo --version


# renders the documentation
@doc:
    cargo doc --no-deps --workspace

# builds the man pages
@man:
    mkdir -p "${CARGO_TARGET_DIR:-target}/man"
    pandoc --standalone -f markdown -t man man/hexit.1.md > "${CARGO_TARGET_DIR:-target}/man/hexit.1"
    pandoc --standalone -f markdown -t man man/hexit.5.md > "${CARGO_TARGET_DIR:-target}/man/hexit.5"

# builds and previews the main man page (hexit.1)
@man-1-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/hexit.1"

# builds and previews the syntax man page (hexit.5)
@man-5-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/hexit.5"
