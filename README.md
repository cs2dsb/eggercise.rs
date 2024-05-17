# eggercise.rs

## Dependencies 
* C++ compiler (for wasm-opt). G++ and Clang both work but the g++ version needs to be at least 17 which isn't in some OS apt repos yet. `apt install clang`
* OpenSSL v3.x `apt install libssl-dev`

## TODO
* Solution for leptos routes working offline in the service worker
    * /* -> index?
    * Included in package generated from shared crate?
* ~~Modify service setup so releases are in versioned folders rather than just the main binary (so assets are versioned too)~~ (Update it to docker)
* Improve client-server error situation (see api.rs)
* The service worker mapping unknown URLs to index isn't working in FF

## Docker build
* Cargo sweep
