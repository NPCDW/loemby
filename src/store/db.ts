import Database from '@tauri-apps/plugin-sql';
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useDb = defineStore('db', () => {
    const db = ref<Database>()

    async function init() {
        db.value = await Database.load('sqlite:loemby.db');
    }

    return { init, db }
})
