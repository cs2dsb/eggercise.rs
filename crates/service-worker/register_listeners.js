
self.addEventListener('install', event => {
    const init_then_install = wasm_bindgen('service_worker_bg.wasm')
        .then(_ => wasm_bindgen.worker_install(self))
        .catch(err => console.error(`Error initializing and installing service worker wasm: ${err}`));  
    event.waitUntil(init_then_install);
});

self.addEventListener('activate', event => {
    event.waitUntil(wasm_bindgen.worker_activate(self));
});

self.addEventListener('fetch', event => {
    event.waitUntil(wasm_bindgen.worker_fetch(self, event));
});