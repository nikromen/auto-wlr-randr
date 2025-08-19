default:
    @just --list

build:
    cargo build

test:
    cargo test -- --nocapture

test-full-backtrace:
    RUST_LOG=debug RUST_BACKTRACE=full cargo test -v -- --nocapture

container-build:
    podman build -t auto-wlr-randr-test -f tests/Containerfile .

test-in-container: container-build
    podman run --rm auto-wlr-randr-test cargo test -- --nocapture

test-in-container-full-backtrace: container-build
    podman run --rm -e RUST_LOG=debug -e RUST_BACKTRACE=full auto-wlr-randr-test cargo test -v -- --nocapture

container-shell: container-build
    podman run --rm -it auto-wlr-randr-test

fmt:
    cargo-fmt --all

clean:
    cargo clean
