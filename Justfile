all: build test xtests
all-release: build-release test-release xtests-release

export HEXIT_DEBUG := ""


#----------#
# building #
#----------#

# compile the hexit binary
@build:
    cargo build

# compile the hexit binary (in release mode)
@build-release:
    cargo build --release --verbose

# produce an HTML chart of compilation timings
@build-time:
    cargo +nightly clean
    cargo +nightly build -Z timings


#---------------#
# running tests #
#---------------#

# run tests
@test:
    cargo test --workspace -- --quiet

# run tests (in release mode)
@test-release:
    cargo test --workspace --release --verbose

# run mutation tests
@test-mutation:
    cargo +nightly test    --package hexit-lang --features=hexit-lang/with_mutagen -- --quiet
    cargo +nightly mutagen --package hexit-lang --features=hexit-lang/with_mutagen


#------------------------#
# running extended tests #
#------------------------#

# run extended integration tests
@xtests:
    specsheet xtests/*.toml -O cmd.target.hexit="${CARGO_TARGET_DIR:-../target}/debug/hexit" -shide

# run extended integration tests (in release mode)
@xtests-release:
    specsheet xtests/*.toml -O cmd.target.hexit="${CARGO_TARGET_DIR:-../target}/release/hexit"


#-----------------------#
# code quality and misc #
#-----------------------#

# lint the code
@clippy:
    cargo clippy

# generate a code coverage report using tarpaulin via docker
@coverage-docker:
    docker run --security-opt seccomp=unconfined -v "${PWD}:/volume" xd009642/tarpaulin cargo tarpaulin --workspace --out Html

# update dependency versions, and check for outdated ones
@update-deps:
    cargo update
    just fuzz/update-deps
    command -v cargo-outdated >/dev/null || (echo "cargo-outdated not installed" && exit 1)
    cargo outdated

# list unused dependencies
@unused-deps:
    command -v cargo-udeps >/dev/null || (echo "cargo-udeps not installed" && exit 1)
    cargo +nightly udeps

# print versions of the necessary build tools
@versions:
    rustc --version
    cargo --version


#---------------#
# documentation #
#---------------#

# render the documentation
@doc:
    cargo doc --no-deps --workspace

# build the man pages
@man:
    mkdir -p "${CARGO_TARGET_DIR:-target}/man"
    pandoc --standalone -f markdown -t man man/hexit.1.md > "${CARGO_TARGET_DIR:-target}/man/hexit.1"
    pandoc --standalone -f markdown -t man man/hexit.5.md > "${CARGO_TARGET_DIR:-target}/man/hexit.5"

# build and preview the main man page (hexit.1)
@man-1-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/hexit.1"

# build and preview the syntax man page (hexit.5)
@man-5-preview: man
    man "${CARGO_TARGET_DIR:-target}/man/hexit.5"
