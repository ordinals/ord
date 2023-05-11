cargo build -r --target x86_64-unknown-linux-musl --config target.x86_64-unknown-linux-musl.linker=\"x86_64-unknown-linux-musl-gcc\"
cp cp ./target/x86_64-unknown-linux-musl/release/ord ./release/ord