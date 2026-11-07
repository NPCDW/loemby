import assert from 'node:assert/strict'
import { test, after } from 'node:test'
import { fileURLToPath } from 'node:url'
import { createServer, transformWithOxc } from 'vite'
import { parse, compileScript, compileTemplate } from 'vue/compiler-sfc'
import { createRenderer, nextTick } from 'vue'
import { createPinia } from 'pinia'

const root = fileURLToPath(new URL('../../../', import.meta.url))
const clientVue = {
    name: 'startup-client-vue',
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
const { createRuntimeStartup, createStartupInitializer } = await server.ssrLoadModule('/src/util/startup.ts')
const { default: App } = await server.ssrLoadModule('/src/App.vue')
const { useRuntimeConfig } = await server.ssrLoadModule('/src/store/runtimeConfig.ts')
const config = version => ({ version, app_config: { log_level: 'info', database_type: 'sqlite', database_url: '' }, axum_port: 1234 })
function deferred() {
    let resolve, reject
    const promise = new Promise((yes, no) => { resolve = yes; reject = no })
    return { promise, resolve, reject }
}
async function flush() { for (let i = 0; i < 8; i++) { await Promise.resolve(); await nextTick() } }

test('启动请求等待就绪，合并重复请求并保持成功结果', async () => {
    const task = deferred()
    let calls = 0
    const startup = createRuntimeStartup(() => { calls++; return task.promise })
    const first = startup.getRuntimeConfig()
    assert.equal(startup.getRuntimeConfig(), first)
    assert.equal(startup.startupStatus.value, 'pending')
    assert.equal(startup.runtimeConfig.value, undefined)
    await flush()
    assert.equal(calls, 1)
    task.resolve(config('ready'))
    await first
    assert.equal(startup.startupStatus.value, 'ready')
    assert.equal(startup.runtimeConfig.value.version, 'ready')
    await startup.getRuntimeConfig()
    assert.equal(calls, 1)
})

test('后端拒绝转换为可见错误，重试成功且无未处理拒绝', async () => {
    let calls = 0
    const startup = createRuntimeStartup(async () => {
        if (++calls === 1) throw '数据库初始化失败，请检查配置后重启应用'
        return config('retry')
    })
    await startup.getRuntimeConfig()
    assert.equal(startup.startupStatus.value, 'failed')
    assert.match(startup.startupError.value, /数据库初始化失败/)
    assert.equal(startup.startupFailure.value, 'backend')
    await startup.getRuntimeConfig()
    assert.equal(startup.startupStatus.value, 'ready')
    assert.equal(startup.startupError.value, '')
})

test('未知错误不展示异常内容或凭据', async () => {
    const startup = createRuntimeStartup(async () => { throw new Error('secret database URL') })
    await startup.getRuntimeConfig()
    assert.equal(startup.startupError.value, '应用初始化失败。')
})

test('超时后迟到响应不能覆盖新请求结果', async () => {
    const old = deferred()
    let calls = 0
    const startup = createRuntimeStartup(() => ++calls === 1 ? old.promise : Promise.resolve(config('new')), 10)
    await startup.getRuntimeConfig()
    assert.equal(startup.startupFailure.value, 'timeout')
    assert.equal(startup.runtimeConfig.value, undefined)
    await startup.getRuntimeConfig()
    old.resolve(config('old'))
    await flush()
    assert.equal(startup.runtimeConfig.value.version, 'new')
    assert.equal(startup.startupStatus.value, 'ready')
})

test('迟到拒绝被捕获，不覆盖已成功的重试', async () => {
    const old = deferred()
    let calls = 0
    const startup = createRuntimeStartup(() => ++calls === 1 ? old.promise : Promise.resolve(config('new')), 10)
    await startup.getRuntimeConfig()
    await startup.getRuntimeConfig()
    old.reject(new Error('late'))
    await flush()
    assert.equal(startup.startupStatus.value, 'ready')
    assert.equal(startup.startupError.value, '')
})

test('缓存和监听初始化仅尝试一次，失败被捕获', async () => {
    let calls = 0
    const errors = []
    const previous = console.error
    console.error = message => errors.push(message)
    try {
        const initialize = createStartupInitializer([
            { name: '缓存', run: () => { calls++ } },
            { name: '监听', run: async () => { calls++; throw new Error('secret') } },
        ])
        const first = initialize()
        assert.equal(initialize(), first)
        await first
        await initialize()
        assert.equal(calls, 2)
        assert.equal(errors.length, 1)
        assert.doesNotMatch(errors[0], /secret/)
    } finally { console.error = previous }
})

const renderer = createRenderer({
    createElement: type => ({ type, children: [], props: {} }),
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
function nodes(root) { return [root, ...(root.children || []).flatMap(nodes)] }

test('实际App启动壳显示加载/错误/重试，ready前不挂载业务路由', async () => {
    const pinia = createPinia()
    const runtime = useRuntimeConfig(pinia)
    let routes = 0, retries = 0
    runtime.getRuntimeConfig = async () => { retries++ }
    const app = renderer.createApp(App)
    app.use(pinia)
    app.component('router-view', { setup() { routes++; return () => '业务页面' } })
    const host = { children: [] }
    app.mount(host)
    assert.equal(routes, 0)
    assert.match(nodes(host).map(n => n.text || '').join(' '), /正在初始化应用/)
    assert.equal(nodes(host).find(n => n.type === 'main').props['aria-busy'], true)
    runtime.startupStatus = 'failed'
    runtime.startupError = '数据库初始化失败'
    await flush()
    assert.equal(routes, 0)
    assert.match(nodes(host).find(n => n.props?.role === 'alert').text, /数据库初始化失败/)
    await nodes(host).find(n => n.type === 'button').props.onClick()
    assert.equal(retries, 1)
    runtime.runtimeConfig = config('ready')
    runtime.startupStatus = 'ready'
    await flush()
    assert.equal(routes, 1)
    assert.equal(nodes(host).some(n => n.type === 'main'), false)
    app.unmount()
})
