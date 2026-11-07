import assert from 'node:assert/strict'
import { test, after } from 'node:test'
import { fileURLToPath } from 'node:url'
import { createServer, transformWithOxc } from 'vite'
import { parse, compileScript, compileTemplate } from 'vue/compiler-sfc'
import { createRenderer, h, nextTick, ref } from 'vue'
import { createRouter, createMemoryHistory } from 'vue-router'

const root = fileURLToPath(new URL('../../../', import.meta.url))
const clientVue = {
    name: 'test-client-vue',
    async transform(source, filename) {
        if (!filename.endsWith('.vue')) return
        const { descriptor } = parse(source, { filename })
        const script = compileScript(descriptor, { id: filename, genDefaultAs: '_sfc_main' })
        const template = compileTemplate({ source: descriptor.template.content, filename, id: filename, compilerOptions: { bindingMetadata: script.bindings } })
        assert.deepEqual(template.errors, [])
        return transformWithOxc(`${script.content}\n${template.code}\n_sfc_main.render = render; export default _sfc_main`, filename, { lang: 'ts' })
    },
}
const server = await createServer({ root, configFile: false, plugins: [clientVue], optimizeDeps: { noDiscovery: true, include: [], entries: [] }, server: { middlewareMode: true, watch: null, ws: false } })
after(() => server.close())
const { useResource } = await server.ssrLoadModule('/src/android/composables/useResource.ts')
const { useMediaActions } = await server.ssrLoadModule('/src/android/composables/useMediaActions.ts')
const { browseDataKey } = await server.ssrLoadModule('/src/android/data/browse.ts')
const { playbackDataKey } = await server.ssrLoadModule('/src/android/data/playback.ts')
const { platformDataKey } = await server.ssrLoadModule('/src/android/data/platform.ts')
const testPlatform = { versionPolicy: async () => 'high-resolution', logo: async () => undefined, openUrl: async () => {} }
const { previewBrowseData } = await server.ssrLoadModule('/src/android/preview/browse.ts')
const { previewState } = await server.ssrLoadModule('/src/android/preview/data.ts')

// 无 DOM 宿主验证实际组件的状态与文本，不替代浏览器布局验收。
const renderer = createRenderer({
    createElement: type => ({ type, children: [], props: {}, addEventListener() {}, removeEventListener() {}, options: [], tagName: type.toUpperCase() }),
    insert(child, parent, anchor) {
        if (child.parent) this.remove(child)
        child.parent = parent
        const index = anchor ? parent.children.indexOf(anchor) : -1
        if (index < 0) parent.children.push(child)
        else parent.children.splice(index, 0, child)
    },
    remove(child) {
        if (child.parent) child.parent.children.splice(child.parent.children.indexOf(child), 1)
        child.parent = null
    },
    patchProp: (node, key, _old, value) => { node.props[key] = value },
    setElementText: (node, text) => { node.text = text; node.children = [] },
    setText: (node, text) => { node.text = text },
    createText: text => ({ text }), createComment: () => ({ text: '' }),
    parentNode: node => node.parent,
    nextSibling: node => node.parent?.children[node.parent.children.indexOf(node) + 1] || null,
})
function deferred() {
    let resolve, reject
    const promise = new Promise((yes, no) => { resolve = yes; reject = no })
    return { promise, resolve, reject }
}
async function flush() { for (let i = 0; i < 8; i++) { await Promise.resolve(); await nextTick() } }
function mountState(setup, data) {
    let state
    const app = renderer.createApp({ setup() { state = setup(); return () => h('div') } })
    if (data) app.provide(browseDataKey, data)
    app.mount({ children: [] })
    return { state, unmount: () => app.unmount() }
}
const user = () => ({ PlayedPercentage: 0, PlaybackPositionTicks: 0, UnplayedItemCount: 0, PlayCount: 0, IsFavorite: false, Played: false })

test('浏览资源：新路由优先，旧成功/失败及卸载后响应无效', async () => {
    const key = ref('a')
    const requests = []
    const { state, unmount } = mountState(() => useResource(() => {
        const task = deferred(); requests.push(task); return task.promise
    }, () => key.value))
    key.value = 'b'
    await flush()
    requests[0].resolve('old')
    await flush()
    assert.equal(state.value.value, undefined)
    assert.equal(state.loading.value, true)
    requests[1].resolve('new')
    await flush()
    assert.equal(state.value.value, 'new')
    const retry = state.load()
    unmount()
    requests[2].reject(new Error('late'))
    await retry
    assert.equal(state.error.value, '')
})

test('浏览资源失败可重试，局部数据保持响应式', async () => {
    let fail = true
    const { state, unmount } = mountState(() => useResource(async () => {
        if (fail) throw new Error('offline')
        return { item: { UserData: user() } }
    }, () => 'route'))
    await flush()
    assert.match(state.error.value, /加载失败/)
    fail = false
    await state.load()
    state.value.value.item.UserData.IsFavorite = true
    assert.equal(state.value.value.item.UserData.IsFavorite, true)
    unmount()
})

test('收藏/已看只写一次并原地更新，重复、缺ID及缺用户状态不提交', async () => {
    const writes = []
    const task = deferred()
    const { state, unmount } = mountState(() => useMediaActions(() => 'server'), {
        favorite: (...args) => { writes.push(args); return task.promise },
    })
    const item = { Id: 'item', UserData: user() }
    const pending = state.favorite(item)
    await state.favorite(item)
    await state.played(item)
    task.resolve({ ...user(), IsFavorite: true })
    await pending
    assert.equal(item.UserData.IsFavorite, true)
    assert.deepEqual(writes, [['server', 'item', true]])
    await state.favorite({ Id: 'missing-user' })
    await state.favorite({ Id: '', UserData: user() })
    assert.equal(writes.length, 1)
    unmount()
})

test('局部标记失败或切换服务器时保留原用户状态', async () => {
    const serverId = ref('one')
    let task = deferred()
    const { state, unmount } = mountState(() => useMediaActions(() => serverId.value), { played: () => task.promise })
    const item = { Id: 'item', UserData: user() }
    let pending = state.played(item)
    task.reject(new Error('offline'))
    await pending
    assert.equal(item.UserData.Played, false)
    assert.match(state.errors.value.item, /更新失败/)
    task = deferred()
    pending = state.played(item)
    serverId.value = 'two'
    task.resolve({ ...user(), Played: true })
    await pending
    assert.equal(item.UserData.Played, false)
    unmount()
})

function playbackAdapter() {
    const calls = []
    const listeners = new Set()
    const adapter = {
        play: async params => { calls.push(params) },
        subscribe(listener) { listeners.add(listener); return () => listeners.delete(listener) },
    }
    return { adapter, calls, listeners, emit: notification => { for (const listener of listeners) listener(notification) } }
}
const stop = (server = 'one', item = 'media') => ({ emby_server_id: server, item_id: item, event: 'stop' })
const media = (Id = 'media', ticks = 123_000_000) => ({
    Id, Name: `测试 ${Id}`, Type: 'Episode', SeriesId: 'series', SeriesName: '测试系列',
    MediaSources: [0, 1].map(index => ({ Id: `source-${index}`, Name: `版本 ${index + 1}`, RunTimeTicks: 1_000_000_000, MediaStreams: [] })),
    UserData: { ...user(), PlaybackPositionTicks: ticks },
})

async function mountPage(name, path, data = previewBrowseData, playback = playbackAdapter().adapter, platform = testPlatform) {
    previewState.delay = 0
    previewState.empty = previewState.failQuery = previewState.failServers = previewState.failUpdate = false
    const Page = (await server.ssrLoadModule(`/src/android/pages/emby/${name}.vue`)).default
    const router = createRouter({ history: createMemoryHistory(), routes: [
        { path: '/nav/emby/:embyId', component: Page },
        { path: '/nav/emby/:embyId/series/:serieId', component: Page },
        { path: '/nav/emby/:embyId/episodes/:episodeId', component: Page },
        { path: '/nav/emby/:embyId/mediaLibrary', component: Page },
        { path: '/nav/emby/:embyId/mediaLibrary/items/:parentId', component: Page },
    ] })
    await router.push(path)
    const container = { children: [] }
    const app = renderer.createApp({ render: () => h(Page) })
    app.provide(browseDataKey, data).provide(playbackDataKey, playback).provide(platformDataKey, platform).use(router).mount(container)
    const state = app._instance.subTree.component.setupState
    const settle = async () => { await new Promise(resolve => setTimeout(resolve, 15)); await flush() }
    await settle()
    const text = node => [node.text || '', ...(node.children || []).map(text)].join(' ')
    const findAll = (predicate, node = container) => [
        ...(predicate(node) ? [node] : []),
        ...(node.children || []).flatMap(child => findAll(predicate, child)),
    ]
    return { state, router, settle, text: () => text(container), findAll, unmount: () => app.unmount() }
}

test('实际系列页：加载文本、特别季切换和分页不调用播放器', async () => {
    const calls = []
    const data = { ...previewBrowseData, episodes: (...args) => { calls.push(args); return previewBrowseData.episodes(...args) } }
    const page = await mountPage('Series', '/nav/emby/demo-home/series/series-0', data)
    assert.match(page.text(), /漫长的旅途/)
    assert.match(page.text(), /选择季/)
    page.state.selectSeason('series-0-season-0')
    await page.settle()
    assert.equal(calls.at(-1)[2], 'series-0-season-0')
    assert.match(page.text(), /番外 1/)
    assert.doesNotMatch(page.text(), /第 1 集 · 当我们启程/)
    page.state.selectSeason('')
    await page.settle()
    page.state.changePage(2)
    await page.settle()
    assert.equal(calls.at(-1)[3], 12)
    page.unmount()
})

test('实际详情页：电影、无媒体源、缺用户状态和路由复用', async () => {
    const page = await mountPage('Episodes', '/nav/emby/demo-home/episodes/episode-1')
    assert.match(page.text(), /电影观看进度/)
    assert.match(page.text(), /媒体版本与轨道信息/)
    assert.match(page.text(), /启动 Android 原生播放器/)
    assert.doesNotMatch(page.text(), /播放（暂未支持）|续播（暂未支持）/)
    assert.equal(page.findAll(node => node.type === 'button' && /播放$/.test(node.text || ''))[0].props.disabled, false)
    assert.equal(page.findAll(node => node.type === 'button' && /下载/.test(node.text || '')).length, 0)
    assert.match(page.text(), /选择章节后/)
    assert.doesNotMatch(page.text(), /下载|章节跳播暂未支持/)
    await page.router.push('/nav/emby/demo-home/episodes/movie-0')
    await page.settle()
    assert.match(page.text(), /暂无媒体源信息/)
    await page.router.push('/nav/emby/demo-cinema/episodes/movie-1')
    await page.settle()
    assert.match(page.text(), /标记操作不可用/)
    assert.equal(page.state.item.Id, 'movie-1')
    page.unmount()
})

test('实际详情播放/续播：自动与手动参数传递，不跳 Vue 播放器页', async () => {
    const playback = playbackAdapter()
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', { ...previewBrowseData, item: async (_server, id) => media(id) }, playback.adapter)
    const button = label => page.findAll(node => node.type === 'button' && node.text === label)[0]
    await button('从头播放').props.onClick()
    assert.deepEqual(playback.calls[0], {
        emby_server_id: 'one', series_id: 'series', item_id: 'media', playback_position_ticks: 0,
        use_direct_link: false, select_policy: 'auto', video_select: 0, audio_select: 0, subtitle_select: 0,
        version_select: 1, download: false,
    })
    page.state.versionIndex = 1
    page.state.manualVersionChange()
    await flush()
    await button('续播').props.onClick()
    assert.deepEqual(playback.calls[1], { ...playback.calls[0], select_policy: 'manual', version_select: 2, playback_position_ticks: 123_000_000 })
    assert.equal(page.router.currentRoute.value.path, '/nav/emby/one/episodes/media')
    page.state.item.UserData.PlaybackPositionTicks = 0
    await flush()
    assert.equal(button('续播'), undefined)
    assert.equal(page.findAll(node => node.type === 'button' && /下载/.test(node.text || '')).length, 0)
    assert.doesNotMatch(page.text(), /下载|章节跳播暂未支持/)
    page.state.item.UserData = undefined
    await page.state.play(true)
    assert.equal(playback.calls.at(-1).playback_position_ticks, 0)
    page.state.item.MediaSources = []
    await page.state.play()
    assert.equal(playback.calls.at(-1).version_select, 1)
    page.unmount()
})

test('实际详情播放：准备中按钮与版本禁用、拒绝重复，失败可见且重试恢复', async () => {
    const playback = playbackAdapter()
    const tasks = []
    playback.adapter.play = params => { playback.calls.push(params); const task = deferred(); tasks.push(task); return task.promise }
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', { ...previewBrowseData, item: async () => media() }, playback.adapter)
    const pending = page.state.play()
    await flush()
    assert.equal(page.state.preparing, true)
    assert.match(page.text(), /正在准备播放/)
    for (const label of ['从头播放', '续播']) {
        assert.equal(page.findAll(node => node.type === 'button' && node.text === label)[0].props.disabled, true)
    }
    assert.equal(page.findAll(node => node.type === 'select')[0].props.disabled, true)
    await page.state.play()
    await page.state.play(true)
    assert.equal(playback.calls.length, 1)
    tasks[0].reject(new Error('native unavailable'))
    await pending
    await flush()
    assert.equal(page.state.preparing, false)
    assert.match(page.text(), /播放准备失败：native unavailable/)
    assert.ok(page.findAll(node => node.props?.role === 'alert').some(node => /native unavailable/.test(node.text)))
    const retry = page.state.play(true)
    assert.equal(page.state.playError, '')
    tasks[1].resolve()
    await retry
    await flush()
    assert.equal(playback.calls.length, 2)
    assert.equal(page.state.preparing, false)
    assert.doesNotMatch(page.text(), /播放准备失败/)
    page.unmount()
})

test('实际详情 stop：只按服务器/条目刷新 UserData，版本不变；卸载解除监听', async () => {
    const playback = playbackAdapter()
    const reads = []
    const refreshed = deferred()
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', {
        ...previewBrowseData,
        item: (...args) => { reads.push(args); return reads.length === 1 ? Promise.resolve(media()) : refreshed.promise },
    }, playback.adapter)
    assert.equal(playback.listeners.size, 1)
    page.state.versionIndex = 1
    playback.emit(stop('two'))
    playback.emit(stop('one', 'other'))
    playback.emit({ ...stop(), event: 'start' })
    await flush()
    assert.equal(reads.length, 1)
    playback.emit(stop())
    await flush()
    assert.deepEqual(reads, [['one', 'media'], ['one', 'media']])
    assert.equal(page.state.refreshing, true)
    assert.match(page.text(), /正在刷新观看进度/)
    const updated = media('media', 500_000_000)
    updated.UserData.PlayedPercentage = 50
    updated.UserData.IsFavorite = true
    refreshed.resolve(updated)
    await flush()
    assert.equal(page.state.item.UserData.PlaybackPositionTicks, 500_000_000)
    assert.equal(page.state.item.UserData.IsFavorite, true)
    assert.equal(page.state.versionIndex, 1)
    assert.equal(page.state.refreshing, false)
    assert.match(page.text(), /50%/)
    page.unmount()
    assert.equal(playback.listeners.size, 0)
    playback.emit(stop())
    assert.equal(reads.length, 2)
})

test('实际详情 stop：刷新失败保留进度且显示错误，后续通知可恢复', async () => {
    const playback = playbackAdapter()
    let reads = 0
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', {
        ...previewBrowseData, item: async () => {
            reads++
            if (reads === 2) throw new Error('offline')
            return media('media', reads > 2 ? 700_000_000 : 123_000_000)
        },
    }, playback.adapter)
    playback.emit(stop())
    await flush()
    assert.equal(page.state.item.UserData.PlaybackPositionTicks, 123_000_000)
    assert.match(page.text(), /观看进度刷新失败/)
    playback.emit(stop())
    await flush()
    assert.equal(page.state.item.UserData.PlaybackPositionTicks, 700_000_000)
    assert.equal(page.state.refreshError, '')
    page.unmount()
})

test('实际详情路由 session：旧播放成功/失败/finally 不能覆盖 A→B→A 新准备状态', async () => {
    const playback = playbackAdapter()
    const tasks = []
    playback.adapter.play = params => { playback.calls.push(params); const task = deferred(); tasks.push(task); return task.promise }
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', { ...previewBrowseData, item: async (_server, id) => media(id) }, playback.adapter)
    page.state.versionIndex = 1
    const oldFailure = page.state.play()
    await page.router.push('/nav/emby/one/episodes/other')
    await page.settle()
    assert.equal(page.state.preparing, false)
    assert.equal(page.state.versionIndex, 0)
    const oldSuccess = page.state.play()
    await page.router.push('/nav/emby/one/episodes/media')
    await page.settle()
    const current = page.state.play(true)
    tasks[0].reject('old native error')
    await oldFailure
    tasks[1].resolve()
    await oldSuccess
    assert.equal(page.state.preparing, true)
    assert.equal(page.state.playError, '')
    tasks[2].reject('current native error')
    await current
    assert.equal(page.state.preparing, false)
    assert.match(page.state.playError, /current native error/)
    await page.router.push('/nav/emby/two/episodes/media')
    await page.settle()
    assert.equal(page.state.playError, '')
    const afterUnmount = page.state.play()
    page.unmount()
    tasks[3].reject('late after unmount')
    await afterUnmount
    assert.equal(page.state.playError, '')
    assert.equal(playback.listeners.size, 0)
})

test('实际详情 stop 路由隔离：相同 item 的跨服务器旧刷新和乱序响应不可覆盖新进度', async () => {
    const playback = playbackAdapter()
    const requests = []
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', {
        ...previewBrowseData, item: (serverId, id) => {
            const task = deferred()
            requests.push({ serverId, id, task })
            if (requests.length === 1) task.resolve(media(id))
            return task.promise
        },
    }, playback.adapter)
    playback.emit(stop()) // 旧 server one 的刷新
    await page.router.push('/nav/emby/two/episodes/media')
    await flush()
    requests[2].task.resolve(media('media', 200_000_000))
    await flush()
    requests[1].task.resolve(media('media', 900_000_000))
    await flush()
    assert.equal(page.state.item.UserData.PlaybackPositionTicks, 200_000_000)
    playback.emit(stop('one'))
    assert.equal(requests.length, 3)
    playback.emit(stop('two'))
    playback.emit(stop('two'))
    requests[4].task.resolve(media('media', 400_000_000))
    await flush()
    requests[3].task.reject(new Error('stale refresh'))
    await flush()
    assert.equal(page.state.item.UserData.PlaybackPositionTicks, 400_000_000)
    assert.equal(page.state.refreshError, '')
    playback.emit(stop('two'))
    const currentItem = page.state.item
    page.unmount()
    requests[5].task.resolve(media('media', 990_000_000))
    await flush()
    assert.equal(currentItem.UserData.PlaybackPositionTicks, 400_000_000)
    assert.equal(playback.listeners.size, 0)
})

test('详情选项：高分辨率预选、按类型编号、关闭轨道、直链清除和章节起播', async () => {
    const playback = playbackAdapter()
    const page = await mountPage('Episodes', '/nav/emby/demo-home/episodes/episode-0', previewBrowseData, playback.adapter)
    assert.equal(page.state.versionIndex, 1)
    assert.equal(page.state.rememberSelect, false)
    assert.equal(page.state.subtitleSelect, 2)
    page.state.selectTrack('Audio', { target: { value: '2' } })
    page.state.selectTrack('Subtitle', { target: { value: '-1' } })
    page.state.useDirectLink = true
    await page.state.play(true)
    assert.equal(playback.calls[0].select_policy, 'manual')
    assert.equal(playback.calls[0].audio_select, 2) // 不使用 Emby 原始 Index=4
    assert.equal(playback.calls[0].subtitle_select, -1)
    assert.equal(playback.calls[0].use_direct_link, true)
    page.state.versionIndex = 0
    page.state.manualVersionChange()
    await flush()
    assert.equal(page.state.useDirectLink, false)
    assert.equal(page.state.supportDirectLink, false)
    const chapters = page.findAll(node => node.type === 'button' && node.props.class === 'chapter-button')
    assert.equal(chapters.length, 3)
    await chapters[1].props.onClick()
    assert.equal(playback.calls.at(-1).playback_position_ticks, 9_000_000_000)
    assert.equal(playback.calls.at(-1).download, false)
    page.state.rememberSelect = false
    page.state.selectionModeChange()
    assert.equal(page.state.versionIndex, 1)
    assert.equal(page.state.subtitleSelect, 2)
    page.unmount()
})

test('详情 query：先恢复版本再校验轨道，连播仅跟随同服务器同系列并携带选项', async () => {
    const playback = playbackAdapter()
    const query = '?rememberSelect=true&versionSelect=1&audioSelect=2&subtitleSelect=-1&videoSelect=99&useDirectLink=true&season=series-0-season-1&page=2'
    const page = await mountPage('Episodes', '/nav/emby/demo-home/episodes/episode-0' + query, previewBrowseData, playback.adapter)
    assert.equal(page.state.versionIndex, 0)
    assert.equal(page.state.audioSelect, 2)
    assert.equal(page.state.videoSelect, 1)
    assert.equal(page.state.subtitleSelect, -1)
    assert.equal(page.state.useDirectLink, false)
    assert.deepEqual(page.state.seriesPath.query, { season: 'series-0-season-1', page: '2' })
    playback.emit({ event: 'start', emby_server_id: 'other', series_id: 'series-0', item_id: 'episode-2' })
    playback.emit({ event: 'start', emby_server_id: 'demo-home', series_id: 'other', item_id: 'episode-2' })
    await flush()
    assert.equal(page.state.item.Id, 'episode-0')
    playback.emit({ event: 'start', emby_server_id: 'demo-home', series_id: 'series-0', item_id: 'episode-2' })
    await page.settle()
    assert.equal(page.state.item.Id, 'episode-2')
    assert.equal(page.router.currentRoute.value.query.audioSelect, '2')
    assert.equal(page.router.currentRoute.value.query.page, '2')
    assert.equal(page.state.subtitleSelect, -1)
    page.unmount()
})

test('详情自动版本策略：高码率沿用桌面大小排序，延迟策略不覆盖手动选择', async () => {
    const source = (Id, Size, Width, Height) => ({ Id, Name: Id, Size, MediaStreams: [{ Type: 'Video', Width, Height, IsDefault: true }] })
    const item = { ...media(), MediaSources: [source('large-hd', 900, 1920, 1080), source('small-4k', 500, 3840, 2160)] }
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', { ...previewBrowseData, item: async () => item }, playbackAdapter().adapter, { ...testPlatform, versionPolicy: async () => 'high-bitrate' })
    assert.equal(page.state.versionIndex, 0)
    page.unmount()
    const policy = deferred()
    const next = await mountPage('Episodes', '/nav/emby/one/episodes/media', { ...previewBrowseData, item: async () => item }, playbackAdapter().adapter, { ...testPlatform, versionPolicy: () => policy.promise })
    next.state.versionIndex = 1
    next.state.manualVersionChange()
    policy.resolve('high-bitrate')
    await flush()
    assert.equal(next.state.versionIndex, 1)
    next.unmount()
})

test('详情进度刷新与标记并发：旧刷新不覆盖新收藏，提供显式重试', async () => {
    const playback = playbackAdapter()
    const stale = deferred()
    let reads = 0
    const data = { ...previewBrowseData, item: async () => {
        reads++
        if (reads === 2) return stale.promise
        const result = media('media', reads > 2 ? 500_000_000 : 123_000_000)
        result.UserData.IsFavorite = reads > 2
        return result
    }, favorite: async () => ({ ...user(), IsFavorite: true }) }
    const page = await mountPage('Episodes', '/nav/emby/one/episodes/media', data, playback.adapter)
    playback.emit(stop())
    await flush()
    await page.state.favorite(page.state.item)
    stale.resolve(media())
    await flush()
    assert.equal(page.state.item.UserData.IsFavorite, true)
    assert.match(page.state.refreshError, /重试/)
    const retry = page.findAll(node => node.type === 'button' && node.text === '重试刷新进度')[0]
    await retry.props.onClick()
    await flush()
    assert.equal(page.state.item.UserData.PlaybackPositionTicks, 500_000_000)
    assert.equal(page.state.refreshError, '')
    page.unmount()
})

test('系列页：恢复季与页码、季和单集标记、返回全部剧集保留空季', async () => {
    const page = await mountPage('Series', '/nav/emby/demo-home/series/series-0?season=series-0-season-1&page=2')
    assert.equal(page.state.seasonId, 'series-0-season-1')
    assert.equal(page.state.page, 2)
    const season = page.state.selectedSeason
    assert.ok(season)
    await page.state.favorite(season)
    assert.equal(season.UserData.IsFavorite, false) // fixture 原始为收藏
    await page.state.markSeasonPlayed()
    await page.settle()
    assert.equal(page.state.selectedSeason.UserData.Played, false)
    const episode = page.state.episodes.Items[0]
    await page.state.played(episode)
    assert.equal(episode.UserData.Played, true)
    assert.equal(episode.UserData.PlaybackPositionTicks, 0)
    page.state.changePage(1)
    await page.settle()
    assert.equal(page.router.currentRoute.value.query.page, undefined)
    page.state.selectSeason('')
    await page.settle()
    assert.equal(page.state.seasonId, '')
    assert.doesNotMatch(page.text(), /暂未支持播放|下载/)
    page.unmount()
    const detail = await mountPage('Episodes', '/nav/emby/demo-home/episodes/episode-0?season=&page=2')
    assert.equal(detail.state.seriesPath.query.season, '')
    detail.unmount()
})

test('外部资料：展示ID，拒绝不安全协议并显示打开失败', async () => {
    const opened = []
    const page = await mountPage('Episodes', '/nav/emby/demo-home/episodes/episode-1', previewBrowseData, playbackAdapter().adapter, {
        ...testPlatform, openUrl: async url => { opened.push(url); throw new Error('native unavailable') },
    })
    assert.match(page.text(), /tt1234567/)
    const button = page.findAll(node => node.type === 'button' && node.text === 'IMDb')[0]
    await button.props.onClick()
    await flush()
    assert.deepEqual(opened, ['https://www.imdb.com/title/tt1234567/'])
    assert.match(page.text(), /无法打开外部资料/)
    page.state.item.ExternalUrls.push({ Name: '危险链接', Url: 'javascript:alert(1)' })
    await flush()
    await page.findAll(node => node.type === 'button' && node.text === '危险链接')[0].props.onClick()
    assert.equal(opened.length, 1)
    page.unmount()
})

test('资料与Logo异步隔离：旧请求失败不影响新媒体，无效章节不可点击', async () => {
    const oldOpen = deferred()
    const oldLogo = deferred()
    const platform = { ...testPlatform,
        openUrl: () => oldOpen.promise,
        logo: async (_server, item) => item.Id === 'episode-1' ? oldLogo.promise : 'data:image/png;base64,new-logo',
    }
    const page = await mountPage('Episodes', '/nav/emby/demo-home/episodes/episode-1', previewBrowseData, playbackAdapter().adapter, platform)
    const opening = page.findAll(node => node.type === 'button' && node.text === 'IMDb')[0].props.onClick()
    await page.router.push('/nav/emby/demo-home/episodes/episode-2')
    await page.settle()
    oldOpen.reject(new Error('old failure'))
    oldLogo.resolve('data:image/png;base64,old-logo')
    await opening
    await flush()
    assert.doesNotMatch(page.text(), /无法打开外部资料/)
    const logos = page.findAll(node => node.type === 'img' && node.props.class === 'detail-logo')
    assert.equal(logos.length, 1)
    assert.equal(logos[0].props.src, 'data:image/png;base64,new-logo')
    page.state.item.Chapters.push({ Name: '损坏章节', StartPositionTicks: -1, ChapterIndex: 99 })
    await flush()
    const chapters = page.findAll(node => node.type === 'button' && node.props.class === 'chapter-button')
    assert.equal(chapters.at(-1).props.disabled, true)
    page.unmount()
})

test('实际库内页：翻页后进入子合集复位并加载新parentId', async () => {
    const calls = []
    const data = { ...previewBrowseData, children: (...args) => { calls.push(args); return previewBrowseData.children(...args) } }
    const page = await mountPage('MediaLibraryItems', '/nav/emby/demo-home/mediaLibrary/items/library-movies', data)
    page.state.changePage(2)
    await page.settle()
    assert.equal(calls.at(-1)[2], 18)
    await page.router.push('/nav/emby/demo-home/mediaLibrary/items/collection-specials')
    await page.settle()
    assert.equal(page.state.page, 1)
    assert.equal(calls.at(-1)[1], 'collection-specials')
    assert.equal(calls.at(-1)[2], 0)
    assert.match(page.text(), /番外与幕后/)
    page.unmount()
})

test('实际媒体库页：入口与最新内容独立加载，服务器切换不串数据', async () => {
    const calls = []
    const data = { ...previewBrowseData, libraries: (...args) => {
        calls.push(args)
        return previewBrowseData.libraries(...args)
    } }
    const page = await mountPage('MediaLibrary', '/nav/emby/demo-home/mediaLibrary', data)
    assert.match(page.text(), /全部媒体库/)
    assert.match(page.text(), /电影库 · 分页测试/)
    assert.match(page.text(), /最新内容/)
    await page.router.push('/nav/emby/demo-cinema/mediaLibrary')
    await page.settle()
    assert.deepEqual(calls.map(args => args[0]), ['demo-home', 'demo-cinema'])
    assert.match(page.text(), /电影收藏/)
    page.unmount()
})

test('实际首页：隐藏/恢复只写不读，标签和服务器切换正确请求', async () => {
    let reads = 0
    const writes = []
    const data = { ...previewBrowseData, resume: (...args) => { reads++; return previewBrowseData.resume(...args) }, hideResume: async (...args) => { writes.push(args) } }
    const page = await mountPage('Home', '/nav/emby/demo-home', data)
    const item = page.state.resumeItems[0]
    assert.ok(item)
    await page.state.toggleResume(item)
    await page.settle()
    assert.match(page.text(), /已隐藏，可恢复/)
    await page.state.toggleResume(item)
    assert.equal(reads, 1)
    assert.deepEqual(writes.map(args => args[2]), [true, false])
    page.state.activeTab = 'count'
    await page.settle()
    assert.match(page.text(), /电视剧/)
    await page.router.push('/nav/emby/demo-cinema')
    await page.settle()
    assert.equal(page.state.serverId, 'demo-cinema')
    page.unmount()
})
