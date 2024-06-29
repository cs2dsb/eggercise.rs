
self.version = 'SERVICE_WORKER_VERSION';

self.addEventListener('install', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_install(self, self.version))
        .catch(err => console.error(`Error initializing or installing service worker wasm: ${err}`)));
});

self.addEventListener('activate', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_activate(self, self.version))
        .catch(err => console.error(`Error initializing or activating service worker wasm: ${err}`)));
});

self.addEventListener('push', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_push(self, self.version, event))
        .catch(err => console.error(`Error initializing or calling worker_push: ${err}`)));
});

self.addEventListener('pushsubscriptionchange', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_push_subscription_change(self, self.version, event))
        .catch(err => console.error(`Error initializing or calling worker_push_subscription_change: ${err}`)));
});

self.addEventListener('notificationclick', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_notification_click(self, self.version, event))
        .catch(err => console.error(`Error initializing or calling worker_notification_click: ${err}`)));
});


// TODO:
// self.addEventListener('fetch', event => {
//     event.waitUntil(self.init
//         .then(_ => wasm_bindgen.worker_fetch(self, self.version, event))
//         .catch(err => console.error(`Error initializing or calling worker_message: ${err}`)));
// });

self.addEventListener('message', event => {
    event.waitUntil(self.init
        .then(_ => wasm_bindgen.worker_message(self, event))
        .catch(err => console.error(`Error initializing or calling worker_message: ${err}`)));
});

// Done like this because the JS part of the service worker is always available but the wasm part, if
// loaded traditionally won't be. To make it work that way would require reimplementing the caching 
// that the wasm is doing here in JS. Promise is stashed on self so install & activate can await it
self.init = wasm_bindgen('data:application/wasm;base64,SERVICE_WORKER_BASE64');