
// self.version = 'SERVICE_WORKER_VERSION';

// self.addEventListener('install', event => {
//     event.waitUntil(self.init
//         .then(_ => wasm_bindgen.worker_install(self, self.version))
//         .catch(err => console.error(`Error initializing and installing service worker wasm: ${err}`)));
// });

// self.addEventListener('activate', event => {
//     event.waitUntil(self.init
//         .then(_ => wasm_bindgen.worker_activate(self, self.version))
//         .catch(err => console.error(`Error initializing and activating service worker wasm: ${err}`)));
// });

// self.addEventListener('push', event => {
//     event.waitUntil(wasm_bindgen.worker_push(self, self.version, event));
// });

// self.addEventListener('pushsubscriptionchange', event => {
//     event.waitUntil(wasm_bindgen.worker_push_subscription_change(self, self.version, event));
// });

// TODO:
// self.addEventListener('fetch', event => {
//     event.waitUntil(wasm_bindgen.worker_fetch(self, self.version, event));
// });


// self.addEventListener('message', event => {
//     wasm_bindgen.worker_message(self, event);
// });

// Done like this because the JS part of the service worker is always available but the wasm part, if
// loaded traditionally won't be. To make it work that way would require reimplementing the caching 
// that the wasm is doing here in JS. Promise is stashed on self so install & activate can await it
// self.init = wasm_bindgen('data:application/wasm;base64,S-E-R-V-I-C-E_WORKER_BASE64');

// Listen to `push` notification event. Define the text to be displayed
// and show the notification.
self.addEventListener('push', function(event) {
    const data = event.data?.text() || "No data provided";
    event.waitUntil(self.registration.showNotification('ServiceWorker Cookbook', {
        body: data
    }));
});
  
// Listen to  `pushsubscriptionchange` event which is fired when
// subscription expires. Subscribe again and register the new subscription
// in the server by sending a POST request with endpoint. Real world
// application would probably use also user identification.
self.addEventListener('pushsubscriptionchange', function(event) {
    console.log('Subscription expired');
    event.waitUntil(
        self.registration.pushManager
            .subscribe({ userVisibleOnly: true })
            .then(function(subscription) {
                console.log('Subscribed after expiration', subscription.endpoint);
                return fetch('register', {
                    method: 'post',
                    headers: {
                        'Content-type': 'application/json'
                    },
                    body: JSON.stringify({
                        endpoint: subscription.endpoint
                    })
                });
            })
    );
});