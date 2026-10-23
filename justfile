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
    cargo test

# Serve the browser playground with live reload
[working-directory: 'app']
serve:
    trunk serve --open

# Build the browser playground into app/dist. Pass the path it is served under.
[working-directory: 'app']
web url="/":
    trunk build --release --public-url {{url}}
