import assert from 'node:assert/strict'
import { test, after } from 'node:test'
import { execFileSync } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'
import { createServer } from 'vite'
import vue from '@vitejs/plugin-vue'
import { renderToString } from 'vue/server-renderer'
import { createRenderer, createSSRApp, h, nextTick } from 'vue'
import * as VueRouter from 'vue-router'
import { build } from 'esbuild'

const root = fileURLToPath(new URL('../../../', import.meta.url))
const server = await createServer({
    root, configFile: false, plugins: [vue()],
    optimizeDeps: { noDiscovery: true, include: [], entries: [] },
    server: { middlewareMode: true, watch: null, ws: false },
})
const { useHistory } = await server.ssrLoadModule('/src/android/composables/useHistory.ts')
const { isAndroidPlatform } = await server.ssrLoadModule('/src/android/platform.ts')
after(() => server.close())

// 无 DOM 的 Vue 宿主，仅测试生命周期与数据状态；不代替浏览器布局验证。
const renderer = createRenderer({
    createElement: type => ({ type, children: [] }),
    insert: (child, parent) => parent.children.push(child),
    remove() {}, patchProp() {}, setElementText() {}, setText() {},
    createText: text => ({ text }), createComment: text => ({ text }),
    parentNode: () => null, nextSibling: () => null,
})
function mount(data) {
    let state
    const app = renderer.createApp({ setup() { state = useHistory(data); return () => h('div') } })
    app.mount({ children: [] })
    return { state, unmount: () => app.unmount() }
}
function deferred() {
    let resolve, reject
    const promise = new Promise((res, rej) => { resolve = res; reject = rej })
    return { promise, resolve, reject }
}
async function flush() { await Promise.resolve(); await Promise.resolve(); await nextTick() }
function api(overrides = {}) {
    return { listServers: async () => [], pageHistory: async () => [0, []], updateHistory: async () => 1, ...overrides }
}

test('仅根据构建平台判断 Android，不使用屏宽、UA 或预览参数', () => {
    assert.equal(isAndroidPlatform({}), false)
    assert.equal(isAndroidPlatform({ TAURI_ENV_PLATFORM: 'android' }), true)
    assert.equal(isAndroidPlatform({ TAURI_PLATFORM: 'android' }), true)
    assert.equal(isAndroidPlatform({ TAURI_ENV_PLATFORM: 'linux', TAURI_PLATFORM: 'android' }), false)
    assert.equal(isAndroidPlatform({ VITE_ANDROID_PREVIEW: 'true' }), false)
})

test('新查询优先，旧响应不能覆盖结果或结束新请求的加载状态', async () => {
    const requests = []
    const { state, unmount } = mount(api({ pageHistory: query => {
        const request = deferred(); requests.push({ ...request, query }); return request.promise
    } }))
    assert.equal(state.loading.value, true)
    state.query.value.page_number = 3
    state.filters.value.series_name = '  星空  '
    const latest = state.applyFilters()
    assert.equal(requests[1].query.page_number, 1)
    assert.equal(requests[1].query.series_name, '星空')
    requests[0].resolve([99, [{ id: 'old' }]])
    await flush()
    assert.equal(state.loading.value, true)
    requests[1].resolve([1, [{ id: 'new' }]])
    await latest
    assert.equal(state.list.value[0].id, 'new')
    assert.equal(state.total.value, 1)
    assert.equal(state.loading.value, false)
    unmount()
})

test('过期失败不影响新结果，卸载后的响应不更新状态', async () => {
    const requests = []
    const { state, unmount } = mount(api({ pageHistory: () => {
        const request = deferred(); requests.push(request); return request.promise
    } }))
    const latest = state.applyFilters()
    requests[1].resolve([1, [{ id: 'new' }]])
    await latest
    requests[0].reject(new Error('old failure'))
    await flush()
    assert.equal(state.error.value, '')
    const pending = state.loadHistory()
    unmount()
    requests[2].resolve([9, [{ id: 'after-unmount' }]])
    await pending
    assert.equal(state.list.value[0].id, 'new')
})

test('查询失败可重试，空记录与清空筛选均可恢复', async () => {
    let fail = true
    const { state, unmount } = mount(api({ pageHistory: async () => {
        if (fail) throw new Error('offline')
        return [0, []]
    } }))
    await flush()
    assert.match(state.error.value, /加载失败/)
    fail = false
    await state.loadHistory()
    assert.equal(state.error.value, '')
    assert.equal(state.list.value.length, 0)
    state.filters.value = { emby_server_id: 'one', series_name: 'series', item_name: 'item' }
    await state.resetFilters()
    assert.equal(state.query.value.page_number, 1)
    assert.equal(state.query.value.emby_server_id, undefined)
    assert.equal(state.query.value.series_name, undefined)
    assert.equal(state.query.value.item_name, undefined)
    unmount()
})

test('分页边界、快速重复分页与记录减少后的末页回退', async () => {
    let count = 25
    const queries = []
    const { state, unmount } = mount(api({ pageHistory: async query => {
        queries.push(query.page_number)
        return [count, [{ id: String(query.page_number) }]]
    } }))
    await flush()
    await state.changePage(3)
    assert.equal(state.query.value.page_number, 3)
    await state.changePage(4)
    assert.equal(queries.length, 2)
    count = 11
    await state.loadHistory()
    assert.equal(state.query.value.page_number, 2)
    assert.equal(state.list.value[0].id, '2')
    await state.changePage(0)
    assert.equal(state.query.value.page_number, 2)
    unmount()
})

test('三种标记状态循环、更新期间防重复、缺失 ID 不提交', async () => {
    const calls = []
    const updates = []
    const { state, unmount } = mount(api({ updateHistory: row => {
        calls.push(row); const request = deferred(); updates.push(request); return request.promise
    } }))
    await flush()
    const row = { id: 'record', pinned: 0 }
    for (const pinned of [1, -1, 0]) {
        const request = state.togglePinned(row)
        assert.equal(state.pendingIds.value.has('record'), true)
        await state.togglePinned(row)
        updates.at(-1).resolve(1)
        await request
        assert.equal(row.pinned, pinned)
        assert.equal(state.pendingIds.value.has('record'), false)
    }
    assert.deepEqual(calls, [{ id: 'record', pinned: 1 }, { id: 'record', pinned: -1 }, { id: 'record', pinned: 0 }])
    await state.togglePinned({ pinned: 0 })
    assert.equal(calls.length, 3)
    unmount()
})

test('更新失败和零受影响行保留原标记，逐条失败可重试', async () => {
    let result = 'throw'
    const { state, unmount } = mount(api({ updateHistory: async () => {
        if (result === 'throw') throw new Error('offline')
        return result
    } }))
    await flush()
    const row = { id: 'record', pinned: 1 }
    await state.togglePinned(row)
    assert.equal(row.pinned, 1)
    assert.match(state.updateErrors.value.record, /更新失败/)
    result = 0
    await state.togglePinned(row)
    assert.equal(row.pinned, 1)
    result = 1
    await state.togglePinned(row)
    assert.equal(row.pinned, -1)
    assert.equal(state.updateErrors.value.record, undefined)
    assert.match(state.notice.value, /未删除/)
    unmount()
})

test('三种标记切换仅更新记录显示，不查询、不换列表、不改分页', async () => {
    let reads = 0
    const { state, unmount } = mount(api({ pageHistory: async () => {
        reads++
        return [25, [{ id: 'record', pinned: 0 }, { id: 'other', pinned: 1 }]]
    } }))
    await flush()
    const list = state.list.value
    const row = list[0]
    for (const pinned of [1, -1, 0]) {
        const task = state.togglePinned(row)
        assert.equal(state.loading.value, false)
        await task
        assert.equal(row.pinned, pinned)
        assert.equal(state.list.value, list)
        assert.deepEqual(list.map(item => item.id), ['record', 'other'])
        assert.equal(state.query.value.page_number, 1)
        assert.equal(state.total.value, 25)
    }
    assert.equal(reads, 1)
    unmount()
})

test('写入成功后旧查询不覆盖标记，新主动查询仍可获取最新服务器状态', async () => {
    const read = deferred()
    let reads = 0
    const { state, unmount } = mount(api({ pageHistory: async () => {
        reads++
        return reads === 2 ? read.promise : [1, [{ id: 'record', pinned: 0 }]]
    } }))
    await flush()
    const oldRow = state.list.value[0]
    const refresh = state.loadHistory()
    await state.togglePinned(oldRow)
    read.resolve([1, [{ id: 'record', pinned: 0 }]])
    await refresh
    assert.equal(state.list.value[0].pinned, 1)
    assert.equal(reads, 2)
    await state.loadHistory()
    assert.equal(state.list.value[0].pinned, 0)
    unmount()
})

test('服务器加载失败可独立重试，不阻塞历史；按顺序排除无 ID 项', async () => {
    let fail = true
    const { state, unmount } = mount(api({ listServers: async () => {
        if (fail) throw new Error('offline')
        return [{ id: 'second', order_by: 2 }, { server_name: 'missing' }, { id: 'first', order_by: 1 }]
    } }))
    await flush()
    assert.match(state.serversError.value, /服务器列表加载失败/)
    assert.equal(state.error.value, '')
    fail = false
    await state.loadServers()
    assert.deepEqual(state.servers.value.map(row => row.id), ['first', 'second'])
    assert.equal(state.serversError.value, '')
    unmount()
})

async function loadRouter(platform, source) {
    const bundled = await build({
        absWorkingDir: root, bundle: true, write: false, format: 'esm', platform: 'node',
        stdin: { contents: source, resolveDir: resolve(root, 'src/router'), sourcefile: 'router.ts', loader: 'ts' },
        define: { 'import.meta.env': JSON.stringify({ BASE_URL: '/', TAURI_ENV_PLATFORM: platform }) },
        plugins: [{ name: 'route-component-stubs', setup(build) {
            build.onResolve({ filter: /\.vue$/ }, args => ({ path: resolve(args.resolveDir, args.path), namespace: 'component' }))
            build.onLoad({ filter: /.*/, namespace: 'component' }, args => ({ contents: `export default { name: ${JSON.stringify(args.path)} }`, loader: 'js' }))
            build.onResolve({ filter: /^vue-router$/ }, () => ({ path: 'router', namespace: 'framework' }))
            build.onLoad({ filter: /.*/, namespace: 'framework' }, () => ({ contents: 'export const createRouter = globalThis.__historyTestRouter.createRouter; export const createWebHistory = globalThis.__historyTestRouter.createMemoryHistory', loader: 'js' }))
        } }],
    })
    globalThis.__historyTestRouter = VueRouter
    const loaded = await import(`data:text/javascript;base64,${Buffer.from(bundled.outputFiles[0].text).toString('base64')}`)
    delete globalThis.__historyTestRouter
    return loaded.default
}

test('实际路由表：Android 浏览主链路使用移动布局，桌面和未适配路由保持原样', async () => {
    const { readFile } = await import('node:fs/promises')
    const current = await readFile(resolve(root, 'src/router/router.ts'), 'utf8')
    const baseline = execFileSync('git', ['show', 'HEAD:src/router/router.ts'], { cwd: root, encoding: 'utf8' })
    const desktop = await loadRouter('linux', current)
    const android = await loadRouter('android', current)
    const original = await loadRouter('linux', baseline)
    assert.equal(desktop.resolve('/nav/history').matched.length, 2)
    assert.match(desktop.resolve('/nav/history').matched[1].components.default.name, /src\/page\/History.vue$/)
    const mobileHistory = android.resolve('/nav/history').matched
    assert.equal(mobileHistory.length, 2)
    assert.match((await mobileHistory[0].components.default()).default.name, /src\/android\/Layout.vue$/)
    assert.match((await mobileHistory[1].components.default()).default.name, /src\/android\/pages\/History.vue$/)
    const paths = [
        '/nav/search', '/nav/setting', '/nav/emby/server',
        '/nav/emby/server/episodes/item', '/nav/emby/server/series/series',
        '/nav/emby/server/mediaLibrary', '/nav/emby/server/mediaLibrary/items/parent', '/nav/emby/server/search',
    ]
    const signature = (router, path) => router.resolve(path).matched.map(record => ({ path: record.path, meta: record.meta, name: record.components.default.name }))
    for (const path of paths) {
        assert.deepEqual(signature(desktop, path), signature(original, path))
        if (['/nav/search', '/nav/setting', '/nav/emby/server/search'].includes(path)) {
            assert.deepEqual(signature(android, path), signature(original, path))
        } else {
            const records = android.resolve(path).matched
            assert.equal(records.length, 2)
            assert.match((await records[0].components.default()).default.name, /src\/android\/Layout.vue$/)
            assert.match((await records[1].components.default()).default.name, /src\/android\/pages\/emby\//)
        }
    }
    assert.equal(desktop.getRoutes().find(row => row.path === '/').redirect, '/nav/history')
    assert.equal(android.getRoutes().find(row => row.path === '/').redirect, '/nav/history')
})

test('页面内容：电影、超长名称、缺失 ID、时长和更新反馈', async () => {
    const History = (await server.ssrLoadModule('/src/android/pages/History.vue')).default
    const { historyDataKey } = await server.ssrLoadModule('/src/android/data/history.ts')
    // 显式捕获 setup 状态后给 SSR 填充场景；这里只验证标记内容，不判断布局。
    let setupState
    const Component = {
        ...History,
        name: 'AndroidHistoryTest',
        setup(props, context) { setupState = History.setup(props, context); return setupState },
    }
    const rows = [
        { id: 'movie', emby_server_id: 'server one', item_id: 'film/one', item_type: 'Movie', item_name: '海边的夜晚', played_duration: 7203, pinned: 0 },
        { series_name: '无 ID 的历史', item_name: '无法更新的旧记录', played_duration: undefined, pinned: -1 },
        { id: 'long', emby_server_id: 'server', series_id: 'series', item_id: 'episode', series_name: '超长标题'.repeat(40), item_name: 'LongUnbrokenTitle'.repeat(30), played_duration: -1, pinned: 1 },
    ]
    const router = VueRouter.createRouter({ history: VueRouter.createMemoryHistory(), routes: [{ path: '/:pathMatch(.*)*', component: { render: () => null } }] })
    await router.push('/nav/history')
    const app = createSSRApp(Component).provide(historyDataKey, api()).use(router)
    app.mixin({ created() {
        if (this.$options.name === 'AndroidHistoryTest') {
            setupState.list.value = rows
            setupState.total.value = rows.length
            setupState.pendingIds.value.add('movie')
            setupState.updateErrors.value.long = '标记更新失败，记录未更改，请重试。'
        }
    } })
    const html = await renderToString(app)
    assert.match(html, /电影 · 无剧集信息/)
    assert.match(html, /2h0m3s/)
    assert.match(html, /暂无时长/)
    assert.match(html, /缺少记录 ID，无法更新标记/)
    assert.match(html, /更新中…/)
    assert.match(html, /标记更新失败/)
    assert.match(html, /server%20one\/episodes\/film%2Fone/)
    assert.ok(html.includes('超长标题'.repeat(40)))
    assert.ok(html.includes('LongUnbrokenTitle'.repeat(30)))
    assert.equal((html.match(/<article\b/g) ?? []).length, 3)
    assert.doesNotMatch(html, /继续播放|\/undefined/)
})
