export type EmbyProtocol = 'http' | 'https'

export interface EmbyAddress {
    protocol: EmbyProtocol
    hostname: string
    port: string
    path: string
}

export const DEFAULT_EMBY_PATH = '/emby'

export function newEmbyAddress(): EmbyAddress {
    return {protocol: 'http', hostname: '', port: '', path: DEFAULT_EMBY_PATH}
}

export function parseEmbyAddress(value: string, defaultProtocol: EmbyProtocol = 'http'): EmbyAddress | undefined {
    const match = value.trim().match(/^(?:(https?):\/\/)?([^/?#\s\\]+)(\/[^?#\\]*)?(?:[?#].*)?$/i)
    if (!match) return

    const [, scheme, authority, path = '/'] = match
    if (authority.includes('@') || authority.endsWith(':')) return

    const protocol = (scheme?.toLowerCase() ?? defaultProtocol) as EmbyProtocol
    try {
        const url = new URL(`${protocol}://${authority}${path}`)
        return {
            protocol,
            hostname: url.hostname,
            // URL.port omits explicit default ports, such as :443 for HTTPS.
            port: authority.match(/:(\d+)$/)?.[1] ?? '',
            path: url.pathname
        }
    } catch {
        return
    }
}

export function parsePastedEmbyAddress(value: string, protocol: EmbyProtocol): EmbyAddress | undefined {
    const address = parseEmbyAddress(value, protocol)
    if (address?.path === '/') address.path = DEFAULT_EMBY_PATH
    return address
}

export function formatEmbyAddress(address: EmbyAddress): string | undefined {
    const hostname = address.hostname.trim()
    const port = address.port.trim()
    const path = address.path.trim() || DEFAULT_EMBY_PATH
    if (!/^(?:\[[^\]]+\]|[^:/\s?#@\\]+)$/.test(hostname) || /[?#\\]/.test(path)) return
    if (port && (!/^\d+$/.test(port) || Number(port) < 1 || Number(port) > 65535)) return

    const authority = `${hostname}${port ? ':' + port : ''}`
    const parsed = parseEmbyAddress(`${address.protocol}://${authority}/${path.replace(/^\/+/, '')}`)
    if (!parsed) return
    return `${parsed.protocol}://${parsed.hostname}${port ? ':' + port : ''}${parsed.path}`
}
