function buildWsUrl() {
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsHost = window.location.host;

    return `${wsProtocol}//${wsHost}/reload`;
}

// Tracks whether we lost connectivity to the server.
let connectivityLost = false;

function doReload(ws) {
    console.log('Reloading.');
    ws.close();
    window.location.reload(true);
}

function handleReload() {
    const ws = new WebSocket(buildWsUrl());

    ws.onopen = () => {
        console.log('Reload server connected.');

        if (connectivityLost) {
            console.log('Connectivity was previously lost. Reloading page.');
            connectivityLost = false;
            doReload(ws);
        }
    };

    ws.onclose = () => {
        console.log('Reload server disconnected. Will retry in 2 seconds.');
        connectivityLost = true;
        setTimeout(handleReload, 2000);
    };

    ws.onmessage = () => doReload(ws);
}

document.addEventListener('DOMContentLoaded', handleReload);
