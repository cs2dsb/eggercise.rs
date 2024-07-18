import init_client, { start_client } from '../wasm/client.js';
import promiser_factory from './sqlite3-worker1-promiser.mjs'

const registerServiceWorker = async () => {
    if ('serviceWorker' in navigator) {
        function handle_update(registration) {
            console.log(`waiting=${registration.waiting != null}, installing=${registration.installing != null}, active=${registration.active != null}`);
            
            if (registration.waiting && registration.active) {
                console.log('Service worker needs update');
              
                // const button = document.createElement('button');
                // button.textContent = 'Do update';
                // button.onclick = () => registration.waiting.postMessage('SKIP_WAITING');

                // document
                //   .getElementById('update_container')
                //   .appendChild(button);
                registration.waiting.postMessage('SKIP_WAITING');

            } else if (registration.installing) {
                console.log('Service worker installing');
                registration.installing.addEventListener('statechange', () => {
                    handle_update(registration);
                });
            } 
        }

        navigator.serviceWorker.register('../wasm/service_worker.js', { type: 'classic', scope: '/' })
            .then(
                (registration) => {
                    registration.addEventListener('updatefound', () => {
                        console.log('Service worker updatefound');
                        handle_update(registration);
                    });

                    if (!registration.installing && registration.waiting) {
                        // In case we missed the updatefound event
                        handle_update(registration);
                    }
                },
                (error) => { console.error(`Service worker registration failed with error: ${error}`); }
            );

        let refreshing = false;
        // Once the worker updates, refresh the page
        // TODO: disabled while debugging
        // navigator.serviceWorker.addEventListener('controllerchange', () => {
        //     if (!refreshing) {
        //         console.log('Service worker changed, refreshing')
        //         refreshing = true
        //         window.location.reload()
        //     }
        // })
    }
};
registerServiceWorker();

// Clean up the sqlite global
delete window.sqlite3Worker1Promiser;

const config = {
    debug: (...args) => { console.log('Debug: ', args)},
    onunhandled: (...args) => { console.log('Unhandled: ', args)},
};
// Build the sqlite promiser & initialize the wasm
Promise.all([
    promiser_factory(config),
    init_client(),
])
.then(async ([promiser]) => {
    // Open the db
    console.log("open db: ", 
        await promiser('open', { 
            filename: 'egg.sqlite3',
            vfs: 'opfs' }));
    
    // Print the config for debugging
    console.log('config-get: ', 
        await promiser('config-get', {}));
    
    // Start the client, passing it the promiser
    await start_client(promiser);
})
.catch((errors) => {
    console.log(`Error constructing promiser or initializing client: ${errors}`)
});