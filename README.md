# eggercise.rs

## Dependencies 
- C++ compiler (for wasm-opt). G++ and Clang both work but the g++ version needs to be at least 17 which isn't in some OS apt repos yet. `apt install clang`
- OpenSSL v3.x `apt install libssl-dev`

## TODO
- [ ] Solution for leptos routes working offline in the service worker
    - /* -> index?
    - Included in package generated from shared crate?
- [x] ~~Modify service setup so releases are in versioned folders rather than just the main binary (so assets are versioned too)~~ (Update it to docker)
- [x] Improve client-server error situation (see api.rs)
    - It could possibly be simplified. Try using it and see if there is actually any benefit to the enum variants over just returning a string
- [ ] The service worker mapping unknown URLs to index isn't working in FF
- [ ] Macro to generate `all_columns` for Iden structs. Failing that a test that checks against a hard-coded list to prevent it going out of sync

## WebauthN 
- [x] Register
- [x] Login
- [x] Add Key
    - [ ] Add key with authentication given by another device (i.e. allow phone to access it from desktop login session)

## Docker build
- Mount sqlite db 