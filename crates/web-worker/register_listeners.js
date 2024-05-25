
importScripts('/wasm/sqlite3.js');

sqlite3InitModule()
    .then((sqlite3) => {
        console.log('Got sqlite: ', sqlite3);

        if (sqlite3.capi.sqlite3_vfs_find("opfs")) {
            console.log('opfs is available');
        } else {
            console.log('opfs is NOT available');
        }
    });

self.version = 'WEB_WORKER_VERSION';

onmessage = function(e) {
    console.log('Worker: Message received from main script:', e);
    postMessage('Pong');
}

self.init = wasm_bindgen('data:application/wasm;base64,WEB_WORKER_BASE64');