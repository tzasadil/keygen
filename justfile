# List recipes
default:
    @just --list

# Score reference layouts on a corpus: english, czech, german, py
ref corpus="english":
    cargo run --release -- run-ref corpus/{{corpus}}.txt

# Score reference layouts, then search for better ones
search corpus="english":
    cargo run --release -- run corpus/{{corpus}}.txt

test:
    cargo test --workspace

# Prove the scoring core still builds for the browser
wasm-check:
    rustup target add wasm32-unknown-unknown
    cargo build -p keygen-core --target wasm32-unknown-unknown --release
