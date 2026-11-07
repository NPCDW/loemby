import type { InjectionKey } from 'vue'
import type { EmbyServer } from '../../store/db/embyServer'
import type { PagePlayHistoryParam, PlayHistory } from '../../store/db/playHistory'

export interface HistoryData {
    listServers(): Promise<EmbyServer[]>
    pageHistory(query: PagePlayHistoryParam): Promise<[number, PlayHistory[]]>
    updateHistory(history: PlayHistory): Promise<number>
}

export const historyDataKey: InjectionKey<HistoryData> = Symbol('androidHistoryData')
