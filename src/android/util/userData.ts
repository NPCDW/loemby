import type { BaseItem, UserData } from '../../api/embyApi'
import { toRaw } from 'vue'

const revisions = new WeakMap<BaseItem, number>()
export const userDataRevision = (item: BaseItem) => revisions.get(toRaw(item)) || 0
export function markUserDataChanged(item: BaseItem) {
    revisions.set(toRaw(item), userDataRevision(item) + 1)
}
export function mergeUserData(item: BaseItem, response: UserData | undefined, revision: number) {
    if (userDataRevision(item) !== revision) return false
    item.UserData = response ? { ...item.UserData, ...response } : item.UserData
    markUserDataChanged(item)
    return true
}
