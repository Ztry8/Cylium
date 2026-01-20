cargo clean
cargo build --release
shasum -a 256 target/release/cylium > target/release/cylium.sha256