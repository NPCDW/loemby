// vue3-smooth-dnd.d.ts
declare module 'vue3-smooth-dnd' {
    import type { DefineComponent } from 'vue'

    export const Container: DefineComponent<{
        // 根据组件实际支持的props添加类型定义
        orientation?: 'vertical' | 'horizontal'
        behaviour?: 'move' | 'copy'
        dragHandleSelector?: string
    }>

    export const Draggable: DefineComponent<{
        // 根据组件实际支持的props添加类型定义
        tag?: string
        class?: string
    }>
}