
self.version = 'SERVICE_WORKER_VERSION';

self.addEventListener('install', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_install(self, self.version))
        .catch(err => console.error(`Error initializing and installing service worker wasm: ${err}`)));
});

self.addEventListener('activate', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_activate(self, self.version))
        .catch(err => console.error(`Error initializing and activating service worker wasm: ${err}`)));
});

self.addEventListener('fetch', event => {
    event.waitUntil(wasm_bindgen.worker_fetch(self, event));
});

// Done like this because the JS part of the service worker is always available but the wasm part, if
// loaded traditionally won't be. To make it work that way would require reimplementing the caching 
// that the wasm is doing here in JS. Promise is stashed on self so install & activate can await it
self.init = wasm_bindgen('data:application/wasm;base64,SERVICE_WORKER_BASE64');