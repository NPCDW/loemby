export function isAndroidPlatform(env: Record<string, unknown> = import.meta.env) {
    const platform = env.TAURI_ENV_PLATFORM ?? env.TAURI_PLATFORM
    return platform === 'android'
}
