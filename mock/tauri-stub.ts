/**
 * 纯浏览器预览用的 Tauri IPC 桩（仅 `--mode mock` 生效，不进生产包）。
 *
 * 目标不是仿真后端，而是把界面喂饱到能逐页核对排版：
 * 服务器、线路、代理、历史都有几条真实感的数据，媒体库有海报占位图。
 */
const img = (label: string, w = 400, h = 600, hue = 210) =>
    `data:image/svg+xml;utf8,${encodeURIComponent(
        `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">
           <defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
             <stop offset="0" stop-color="hsl(${hue},32%,26%)"/>
             <stop offset="1" stop-color="hsl(${hue + 26},38%,14%)"/>
           </linearGradient></defs>
           <rect width="100%" height="100%" fill="url(#g)"/>
           <text x="50%" y="52%" fill="rgba(232,237,244,.62)" font-family="sans-serif"
                 font-size="${Math.round(w / 9)}" text-anchor="middle">${label}</text>
         </svg>`,
    )}`;

const userData = (over: Record<string, unknown> = {}) => ({
    PlayedPercentage: 0,
    UnplayedItemCount: 0,
    PlaybackPositionTicks: 0,
    PlayCount: 0,
    IsFavorite: false,
    Played: false,
    ...over,
});

const mediaStream = (type: string, title: string, extra: Record<string, unknown> = {}) => ({
    Codec: 'h264',
    DisplayTitle: title,
    DisplayLanguage: type === 'Subtitle' ? 'Chinese (Simplified)' : '',
    Title: title,
    BitRate: 8_000_000,
    Height: 1080,
    Width: 1920,
    Type: type,
    Language: 'chi',
    Index: 1,
    IsDefault: true,
    IsExternal: false,
    ...extra,
});

const mediaSource = (n: number, size: number, res: [number, number], name: string) => ({
    Id: `src-${n}`,
    Name: name,
    RunTimeTicks: 2_700_000_000,
    Size: size,
    Bitrate: 9_400_000,
    DirectStreamUrl: '/Videos/1/stream.mkv',
    IsRemote: n === 2,
    Path: n === 2 ? 'https://remote.example.org/media/movie.mkv' : '/media/library/movie.mkv',
    Container: 'mkv',
    MediaStreams: [
        mediaStream('Video', `${res[0]}x${res[1]} HEVC`, { Width: res[0], Height: res[1] }),
        mediaStream('Audio', 'Chinese AAC 5.1', { Type: 'Audio', Index: 2 }),
        mediaStream('Audio', 'Japanese FLAC 2.0', { Type: 'Audio', Index: 3, IsDefault: false }),
        mediaStream('Subtitle', '简体中文', { Type: 'Subtitle', Index: 4, IsExternal: true }),
        mediaStream('Subtitle', 'English', {
            Type: 'Subtitle',
            Index: 5,
            IsExternal: true,
            IsDefault: false,
            DisplayLanguage: 'English',
        }),
    ],
});

const SERVER_IDS = ['srv-1', 'srv-2', 'srv-3'];

const servers = [
    {
        id: 'srv-1',
        server_name: '家里那台',
        base_url: 'http://192.168.1.2:8096',
        username: 'npc',
        auth_token: 'token-1',
        user_id: 'u1',
        order_by: 1,
        line_id: 'line-1',
        reverse_proxy_id: 'no',
        browse_proxy_id: 'follow',
        play_proxy_id: 'follow',
        disabled: 0,
        keep_alive_days: 14,
        last_playback_time: '2026-09-20T12:00:00Z',
        icon_url: img('EMBY', 64, 64, 200),
    },
    {
        id: 'srv-2',
        server_name: '公网中转',
        base_url: 'https://emby.example.org',
        username: 'npc',
        auth_token: 'token-2',
        user_id: 'u2',
        order_by: 2,
        line_id: 'line-2',
        reverse_proxy_id: 'rev-1',
        browse_proxy_id: 'no',
        play_proxy_id: 'no',
        disabled: 0,
        keep_alive_days: 7,
        last_playback_time: '2026-09-22T01:00:00Z',
    },
    {
        id: 'srv-3',
        server_name: '朋友的服',
        base_url: 'http://10.0.0.9:8096',
        order_by: 3,
        disabled: 1,
        browse_proxy_id: 'no',
        play_proxy_id: 'no',
        reverse_proxy_id: 'no',
    },
];

const lines = [
    {
        id: 'line-1',
        name: '直连',
        emby_server_id: 'srv-1',
        emby_server_name: '家里那台',
        base_url: 'http://192.168.1.2:8096',
        reverse_proxy_id: 'no',
        browse_proxy_id: 'follow',
        play_proxy_id: 'follow',
    },
    {
        id: 'line-1b',
        name: '中转',
        emby_server_id: 'srv-1',
        emby_server_name: '家里那台',
        base_url: 'https://relay.example.org/emby',
        reverse_proxy_id: 'rev-1',
        browse_proxy_id: 'no',
        play_proxy_id: 'no',
    },
    {
        id: 'line-2',
        name: '公网',
        emby_server_id: 'srv-2',
        emby_server_name: '公网中转',
        base_url: 'https://emby.example.org',
        reverse_proxy_id: 'rev-1',
        browse_proxy_id: 'no',
        play_proxy_id: 'no',
    },
];

const proxies = [
    { id: 'proxy-1', name: '香港节点', proxy_type: 'http', addr: 'hk.example.org:7890', username: 'npc', location: '203.0.113.8 HK' },
    { id: 'proxy-2', name: '日本节点', proxy_type: 'socks5', addr: 'jp.example.org:1080' },
];

const reverseProxies = [
    { id: 'rev-1', name: '公网入口', url: 'https://proxy.example.org/', location: '198.51.100.4 JP' },
];

const iconLibraries = [{ id: 'lib-1', name: '官方图标集', url: 'https://example.org/icons.json' }];

const globalConfig: Record<string, string> = {
    global_browse_proxy_id: 'proxy-1',
    global_play_proxy_id: 'no',
    app_proxy_id: 'followBrowse',
    play_version_auto_select_policy: 'high-resolution',
    prefetch_playlist: 'yes',
    play_param_IsPlayback: 'true',
    cache_speed_enabled: 'yes',
    cache_speed_ass_style: '{\\an9\\3c&HA066FD&}',
    mpv_cache_seconds: 30,
    mpv_cache_min_bytes: 24,
    mpv_cache_max_bytes: 512,
    mpv_cache_back_seconds: 10,
    mpv_cache_back_min_bytes: 8,
    mpv_cache_back_max_bytes: 256,
    mpv_args: 'ontop=no\nvolume=130\ndemuxer-max-bytes=512MiB',
    external_mpv_switch: 'off',
    trakt_sync_switch: 'on',
    trakt_username: 'npc',
    trakt_proxy_id: 'followBrowse',
    simkl_sync_switch: 'on',
    simkl_username: '',
    simkl_proxy_id: 'followBrowse',
    yamtrack_sync_switch: 'off',
    yamtrack_sync_url: '',
    yamtrack_proxy_id: 'followBrowse',
    danger_accept_invalid_certs: 'false',
    logStoredDays: 30,
    coverImageStoredDays: 30,
    iconStoredDays: 365,
    disabled_image_cache: 'off',
    disabledImage: 'off',
    global_browse_proxy_id_name: '香港节点',
};

const playHistory = Array.from({ length: 22 }, (_, index) => ({
    id: `h-${index}`,
    emby_server_id: SERVER_IDS[index % 3],
    emby_server_name: servers[index % 3].server_name,
    item_type: 'Episode',
    item_id: `ep-${index}`,
    item_name: `第 ${index + 1} 集 深夜的列车`,
    series_id: `series-${index % 5}`,
    series_name: ['葬送的芙莉莲', '海贼王', '孤独的美食家', '切尔诺贝利', '心灵猎人'][index % 5],
    played_duration: 600 + index * 137,
    pinned: index < 2 ? 1 : 0,
}));

const series = Array.from({ length: 10 }, (_, index) => ({
    Id: `series-${index}`,
    Name: ['葬送的芙莉莲', '海贼王', '孤独的美食家', '切尔诺贝利', '心灵猎人', '瑞克和莫蒂', '黑镜', '西部世界', '怪奇物语', '王冠'][index],
    Type: 'Series',
    ProductionYear: 2019 + (index % 5),
    EndDate: index % 3 === 0 ? '2026-01-01' : '',
    Overview:
        '这是一段用于核对排版的简介文字。它需要足够长，好让换行、行距与最大宽度都能看出效果；同时也要有中文、English 与数字 1080p / 2160p 混排，检查等宽数字的对齐。',
    ImageTags: { Primary: 'x' },
    ProviderIds: { Imdb: 'tt1234567', Tmdb: '12345' },
    ExternalUrls: [
        { Url: 'https://www.imdb.com/title/tt1234567', Name: 'IMDb' },
        { Url: 'https://www.themoviedb.org/tv/12345', Name: 'TMDb' },
        { Url: 'https://trakt.tv/shows/example', Name: 'Trakt' },
    ],
    Genres: ['动画', '奇幻', '冒险'],
    OfficialRating: 'PG-13',
    UserData: userData({
        UnplayedItemCount: (index % 4) * 3,
        IsFavorite: index % 3 === 0,
        PlayedPercentage: index % 2 ? 62 : 0,
    }),
    Chapters: [],
}));

const episodes = Array.from({ length: 30 }, (_, index) => ({
    Id: `ep-${index}`,
    Name: `第 ${index + 1} 集 ${['始まり', '旅途的开始', '雨中的站台', '两个人的晚餐', '归途'][index % 5]}`,
    Type: 'Episode',
    SeriesName: '葬送的芙莉莲',
    SeriesId: 'series-0',
    SeasonId: 'season-1',
    SeasonName: '第 1 季',
    ParentIndexNumber: 1,
    IndexNumber: index + 1,
    ProductionYear: 2024,
    PremiereDate: `2024-0${(index % 9) + 1}-1${index % 9}`,
    Overview: '待用的简介占位。',
    ImageTags: {},
    ProviderIds: {},
    ExternalUrls: [],
    Chapters: [],
    MediaSources: [mediaSource(1, 2_400_000_000, [1920, 1080], 'Web-DL 1080p')],
    UserData: userData({
        PlayedPercentage: index < 4 ? 0 : index < 8 ? 47 : 100,
        Played: index >= 8,
        UnplayedItemCount: 0,
    }),
}));

const seasons = Array.from({ length: 3 }, (_, index) => ({
    Id: `season-${index + 1}`,
    Name: `第 ${index + 1} 季`,
    Type: 'Season',
    SeriesId: 'series-0',
    SeriesName: '葬送的芙莉莲',
    IndexNumber: index + 1,
    ProductionYear: 2023 + index,
    Overview: '这一季的简介。',
    ImageTags: { Primary: 'x' },
    ProviderIds: {},
    ExternalUrls: [],
    Chapters: [],
    UserData: userData({ UnplayedItemCount: index === 0 ? 4 : index === 1 ? 2 : 0 }),
}));

const movieDetail = {
    ...episodes[0],
    Id: 'movie-1',
    Name: '沙丘 2',
    Type: 'Movie',
    SeriesName: '',
    SeriesId: '',
    SeasonId: '',
    ParentIndexNumber: 0,
    IndexNumber: 0,
    MediaSources: [
        mediaSource(1, 12_400_000_000, [1920, 1080], 'BluRay 1080p'),
        mediaSource(2, 48_200_000_000, [3840, 2160], 'BluRay 2160p HDR 远端'),
    ],
    Chapters: [
        { StartPositionTicks: 0, Name: '开场', MarkerType: 'Chapter', ChapterIndex: 0 },
        { StartPositionTicks: 420_000_0000, Name: '第一幕', MarkerType: 'Chapter', ChapterIndex: 1 },
        { StartPositionTicks: 1_800_000_0000, Name: '第二幕', MarkerType: 'Chapter', ChapterIndex: 2 },
        { StartPositionTicks: 4_600_000_0000, Name: '结尾', MarkerType: 'Chapter', ChapterIndex: 3 },
    ],
    UserData: userData({ PlayedPercentage: 63, PlaybackPositionTicks: 1_700_000_0000 }),
};

const continuePlay = episodes.slice(0, 8).map(episode => ({ ...episode, Type: 'Episode' }));

const libraries = [
    { Id: 'lib-movies', Name: '电影', Type: 'CollectionFolder', CollectionType: 'movies', ImageTags: { Primary: 'x' } },
    { Id: 'lib-tv', Name: '剧集', Type: 'CollectionFolder', CollectionType: 'tvshows', ImageTags: { Primary: 'x' } },
    { Id: 'lib-anime', Name: '番剧', Type: 'CollectionFolder', CollectionType: 'tvshows', ImageTags: { Primary: 'x' } },
    { Id: 'lib-music', Name: '音乐', Type: 'CollectionFolder', CollectionType: 'music', ImageTags: { Primary: 'x' } },
    { Id: 'lib-books', Name: '书库', Type: 'CollectionFolder', CollectionType: 'books', ImageTags: { Primary: 'x' } },
];

const searchItems = [...series.slice(0, 6), ...episodes.slice(0, 6)];

function imageUrlFor(itemId: string, imageType: string): string {
    const seed = itemId.split('').reduce((sum, ch) => sum + ch.charCodeAt(0), 0);
    const isWide = imageType === 'Thumb' || imageType === 'Backdrop';
    const hue = 190 + (seed % 90);
    if (imageType === 'Logo') {
        return img('LOGO', 480, 120, hue);
    }
    return isWide ? img('', 640, 360, hue) : img('', 400, 600, hue);
}

const handlers: Record<string, (args: any) => unknown> = {
    get_runtime_config: () => ({
        version: '0.42.6',
        app_config: { log_level: 'info', database_type: 'sqlite', database_url: '/home/npc/.config/loemby/config/app-config.json' },
        axum_port: 17890,
    }),
    get_sys_info: () => 'npc-desktop',

    list_all_emby_server: () => servers,
    get_emby_server: (a) => servers.find(s => s.id === a?.id) ?? null,
    add_emby_server: () => 1,
    update_emby_server: () => 1,
    delete_emby_server: () => 1,
    update_emby_server_order: () => 1,
    defer_emby_server_order: () => 1,

    list_all_emby_line: () => lines,
    list_emby_server_line: (a) => lines.filter(l => l.emby_server_id === a?.embyServerId),
    get_emby_line: (a) => lines.find(l => l.id === a?.id) ?? null,
    add_emby_line: () => 1,
    update_emby_line: () => 1,
    update_line_emby_server_name: () => 1,
    delete_emby_line: () => 1,
    delete_line_by_emby_server_id: () => 1,

    list_all_proxy_server: () => proxies,
    get_proxy_server: (a) => proxies.find(p => p.id === a?.id) ?? null,
    add_proxy_server: () => 1,
    update_proxy_server: () => 1,
    delete_proxy_server: () => 1,

    list_all_reverse_proxy_server: () => reverseProxies,
    get_reverse_proxy_server: (a) => reverseProxies.find(p => p.id === a?.id) ?? null,
    add_reverse_proxy_server: () => 1,
    update_reverse_proxy_server: () => 1,
    delete_reverse_proxy_server: () => 1,

    list_all_emby_icon_library: () => iconLibraries,
    get_emby_icon_library: () => null,
    add_emby_icon_library: () => 1,
    update_emby_icon_library: () => 1,
    delete_emby_icon_library: () => 1,

    list_all_global_config: () => Object.entries(globalConfig).map(([config_key, config_value]) => ({ id: config_key, config_key, config_value })),
    get_global_config: (a) => (globalConfig[a?.configKey] !== undefined ? { id: a.configKey, config_key: a.configKey, config_value: globalConfig[a.configKey] } : null),
    add_global_config: () => 1,
    update_global_config: () => 1,
    delete_global_config: () => 1,

    page_play_history: (a) => {
        const body = a?.body ?? {};
        let rows = playHistory;
        if (body.emby_server_id) rows = rows.filter(r => r.emby_server_id === body.emby_server_id);
        if (body.series_name) rows = rows.filter(r => r.series_name.includes(body.series_name));
        if (body.item_name) rows = rows.filter(r => r.item_name.includes(body.item_name));
        const start = ((body.page_number ?? 1) - 1) * (body.page_size ?? 30);
        return [rows.length, rows.slice(start, start + (body.page_size ?? 30))];
    },
    get_play_history: () => null,
    add_play_history: () => 1,
    update_play_history: () => 1,
    cancel_pinned_play_history: () => 1,

    emby_get_server_info: () => ({ ServerName: '家里那台', Id: 'srv-1' }),
    emby_authenticate_by_name: () => ({ User: { Id: 'u1' }, AccessToken: 'token-1' }),
    emby_logout: () => '',

    emby_search: () => ({ TotalRecordCount: searchItems.length, Items: searchItems }),
    emby_get_continue_play_list: () => ({ TotalRecordCount: continuePlay.length, Items: continuePlay }),
    emby_get_favorite_list: () => ({ TotalRecordCount: series.length, Items: series }),
    emby_next_up: () => ({ TotalRecordCount: episodes.length, Items: episodes.slice(0, 12) }),
    emby_get_media_library_list: () => ({ TotalRecordCount: libraries.length, Items: libraries }),
    emby_get_media_library_child_latest: (a) => (a?.body?.parent_id === 'lib-movies' ? series.slice(0, 12) : episodes.slice(0, 12)),
    emby_get_media_library_child: () => ({ TotalRecordCount: 46, Items: [...series, ...episodes].slice(0, 30) }),
    emby_count: () => ({
        MovieCount: 1284,
        SeriesCount: 236,
        EpisodeCount: 9821,
        GameCount: 0, ArtistCount: 0, ProgramCount: 0, GameSystemCount: 0,
        TrailerCount: 0, SongCount: 0, AlbumCount: 0, MusicVideoCount: 0,
        BoxSetCount: 12, BookCount: 0, ItemCount: 11353,
    }),
    emby_items: (a) => {
        const id = a?.body?.item_id;
        if (id === 'movie-1') return movieDetail;
        if (typeof id === 'string' && id.startsWith('series')) return series.find(s => s.Id === id) ?? series[0];
        return episodes.find(e => e.Id === id) ?? episodes[0];
    },
    emby_seasons: () => ({ TotalRecordCount: seasons.length, Items: seasons }),
    emby_episodes: () => ({ TotalRecordCount: episodes.length, Items: episodes.slice(0, 30) }),
    emby_playback_info: () => ({ PlaySessionId: 'ps-1', MediaSources: [mediaSource(1, 12_400_000_000, [1920, 1080], 'BluRay 1080p')] }),
    emby_star: () => userData({ IsFavorite: true }),
    emby_unstar: () => userData({ IsFavorite: false }),
    emby_played: () => userData({ Played: true, PlayedPercentage: 100 }),
    emby_unplayed: () => userData({ Played: false, PlayedPercentage: 0 }),
    emby_hide_from_resume: () => '',

    app_http_get_proxy_location: () => ({ ip: '203.0.113.8', country_code: 'HK' }),
    app_http_get_reverse_proxy_location: () => ({ ip: '198.51.100.4', country_code: 'JP' }),
    app_http_get_emby_icon_library: () => ({
        name: '官方图标集',
        icons: Array.from({ length: 18 }, (_, i) => ({ name: `图标 ${i + 1}`, url: img(`I${i}`, 64, 64, 150 + i * 12) })),
    }),

    call_player: () => '',
    go_trakt_auth: () => undefined,
    go_simkl_auth: () => undefined,
    open_url: () => undefined,
    updater: () => false,
    restart_app: () => false,
    clean_emby_image_cache: () => undefined,
    clean_icon_cache: () => undefined,
    open_folder: () => undefined,
    open_file: () => undefined,
};

declare global {
    interface Window {
        __TAURI_INTERNALS__: unknown;
        __TAURI_MOCK_DELAY__: number;
    }
}

/**
 * 图片走本地 axum 的 /image 端点，纯浏览器里没有这个服务。
 * 因为端口不同（跨源），拦截必须从 DOM 层入手，不能用 fetch：
 *  - 劫持 img 的 src，指向 axum 的地址时换成本地生成的占位图
 *  - 覆盖 Image 构造函数，覆盖 new Image().src = ... 的用法
 */
const IMAGE_ORIGIN_HINT = '/image/';

function placeholderFor(src: string): string | null {
    if (!src.includes(IMAGE_ORIGIN_HINT)) {
        return null;
    }
    let url: URL;
    try {
        url = new URL(src, location.origin);
    } catch {
        return null;
    }
    if (url.pathname.includes('/image/emby')) {
        return imageUrlFor(url.searchParams.get('item_id') ?? '', url.searchParams.get('image_type') ?? '');
    }
    if (url.pathname.includes('/image/icon')) {
        const raw = url.searchParams.get('image_url') ?? '';
        return raw.startsWith('data:') ? raw : img('ICON', 64, 64, 180);
    }
    return null;
}

const descriptor = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
if (descriptor?.set && descriptor.get) {
    Object.defineProperty(HTMLImageElement.prototype, 'src', {
        configurable: true,
        get() {
            return descriptor.get!.call(this);
        },
        set(value: string) {
            descriptor.set!.call(this, placeholderFor(String(value)) ?? value);
        },
    });
}

const NativeImage = window.Image;
// @ts-expect-error 桩里放宽类型，换成能改写 src 的子类
window.Image = class extends NativeImage {
    constructor(width?: number, height?: number) {
        super(width, height);
    }

    set src(value: string) {
        super.src = placeholderFor(value) ?? value;
    }

    get src(): string {
        return super.src;
    }
};

window.__TAURI_MOCK_DELAY__ = 120;

window.__TAURI_INTERNALS__ = {
    async invoke(command: string, args?: unknown) {
        // 真实后端有网络延迟，这里补一点，方便核对骨架屏
        await new Promise(resolve => setTimeout(resolve, window.__TAURI_MOCK_DELAY__));
        const handler = handlers[command];
        if (!handler) {
            console.warn('[mock-tauri] 未实现命令:', command);
            return null;
        }
        return handler(args);
    },
    transformCallback(callback: unknown) {
        return callback;
    },
    convertFileSrc: (path: string) => path,
};

/* —— 与 @tauri-apps/api 保持一致的最小导出面 —— */

export const invoke = window.__TAURI_INTERNALS__.invoke as (command: string, args?: unknown) => Promise<unknown>;

/** listen 只登记回调，桩里不主动触发任何事件 */
export async function listen(): Promise<() => void> {
    return () => undefined;
}

export async function once(): Promise<() => void> {
    return () => undefined;
}

export async function emit(): Promise<void> {
    return undefined;
}

export function clearMocks(): void {
    return undefined;
}

export default { invoke, listen, once, emit };
