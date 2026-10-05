# Run all CI checks.
check:
    bash scripts/checks/run.sh

# Build and open the public API documentation.
doc:
    cargo clean --doc
    cargo doc --all-features -p thisversion --open --no-deps

# Build and open the book.
book:
    mdbook serve docs

# Run GitHub Actions CI locally.
ci:
    bash scripts/local/ci.sh

# Compute the minimum supported Rust version.
msrv:
    bash scripts/local/msrv.sh

# Clear empty directories.
clear:
    find . -type d -empty \
        -not -path './.git' \
        -not -path './.git/*' \
        -delete

# Save a zip of all the files tracked by git to target/tmp/archive.zip:
archive:
    git archive -o target/tmp/archive.zip HEAD

# Search filenames and file contents for TODOs.
todo:
    bash scripts/local/todo.sh
