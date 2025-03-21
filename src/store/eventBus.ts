import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useEventBus = defineStore('eventBus', () => {
    const eventList = ref<{[name: string]: Function[]}>({})

    function on(name: string, fn: Function) {
        eventList.value[name] = eventList.value[name] || []
        eventList.value[name].push(fn);
    }
    
    function emit(name: string, data: any) {
        if (eventList.value[name]) {
            eventList.value[name].forEach((fn) => fn(data))
        }
    }
    
    function remove(name: string, fn: Function) {
        if (eventList.value[name] && eventList.value[name].indexOf(fn) > -1) {
            eventList.value[name].splice(eventList.value[name].indexOf(fn), 1);
        }
    }

    function off(name: string) {
        if (eventList.value[name]) {
            delete eventList.value[name];
        }
    }

    return { on, emit, off, remove }
})
