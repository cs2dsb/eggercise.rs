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
        navigator.serviceWorker.addEventListener('controllerchange', () => {
            if (!refreshing) {
                console.log('Service worker changed, refreshing')
                refreshing = true
                window.location.reload()
            }
        })
    }
};
registerServiceWorker();

const registerWebWorker = () => {
    if (window.Worker) {
        const worker = new Worker('../wasm/web_worker.js');
        worker.onmessage = (e) => {
            console.log('message from worker: ', e);
        };
      
        worker.postMessage('Ping');
    } else {
        console.log('Your browser doesn\'t support web workers.');
    }
};
registerWebWorker();