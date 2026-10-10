import { ElMessage, ElNotification } from 'element-plus';
import dayjs from 'dayjs';
import router from '../router/router';

/**
 * 同类错误短时间内只提示一次。
 * 数据库连接断开时，页面上并发/轮询的请求会成片失败，不能刷屏。
 */
const recentMessages: {[key: string]: number} = {}
const DEDUPE_WINDOW_MS = 10000

export function dedupeKey(content: string, interval = DEDUPE_WINDOW_MS): boolean {
    const now = dayjs().valueOf()
    const last = recentMessages[content]
    if (last && now - last < interval) {
        return false
    }
    recentMessages[content] = now
    return true
}

/**
 * 统一的请求失败提示。
 * - 数据库不可用：只提示一次并引导到排查页，避免轮询刷屏
 * - 其他错误：10 秒内同类错误去重后提示
 */
export function notifyRequestError(prefix: string, error: unknown) {
    const detail = String(error ?? '')
    const dbUnavailable = /数据库连接失败|数据库连接不可用|Database\(\w+\)|pool timed out|PoolTimedOut/i.test(detail)
    if (dbUnavailable) {
        notifyDbUnavailable()
        return
    }
    const content = prefix + detail
    if (!dedupeKey(content)) {
        return
    }
    ElMessage.error(content)
}

function notifyDbUnavailable() {
    if (!dedupeKey('db-unavailable', 60_000)) {
        return
    }
    ElNotification({
        type: 'error',
        title: '数据库不可用',
        message: '数据库连接已断开，数据读写功能暂时不可用，点击查看详情',
        duration: 8000,
        onClick: () => {
            router.replace({ name: 'fatalError' })
        },
    })
}
