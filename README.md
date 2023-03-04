## Dev

```
cargo clippy --all-features --tests --examples -- -D clippy::all
cargo +nightly clippy --all-features --tests --examples -- -D clippy::all

cargo fmt -- --check

cargo test-all-features -- --nocapture
```

```
RUST_BACKTRACE=1 RUST_LOG=trace cargo run -p linode-api-proxy -- --http-listen-addr 127.0.0.1:8080 -v

export LINODE_API_TOKEN="xxx"
curl -H "Authorization: Bearer $LINODE_API_TOKEN" http://127.0.0.1:8080/v4/profile -v
curl -H "Authorization: Bearer $LINODE_API_TOKEN" 'http://127.0.0.1:8080/v4/linode/instances/show_by_label?label=xxx' -v
```
