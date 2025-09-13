cargo build --release -p device-driver-wasm --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir website/static/wasm target/wasm32-unknown-unknown/release/device_driver_wasm.wasm
