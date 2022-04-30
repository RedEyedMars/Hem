wasm-pack build --release --target web && npm --prefix ../www run build && cp -r ../www/dist .  && cp ./pkg/*_bg.wasm* ./dist && cargo build --manifest-path=../server/Cargo.toml
