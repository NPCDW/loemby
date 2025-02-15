import { fetch } from '@tauri-apps/plugin-http';

async function getServerInfo(base_url: string) {
    return fetch(base_url + '/System/Info/Public', {
        method: 'GET',
    });
}

export default {
    getServerInfo
}