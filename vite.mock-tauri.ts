import { resolve } from 'path';
import type { Plugin } from 'vite';

/** 桩入口的名字，供 build.rollupOptions.input 引用 */
export const MOCK_TAURI_ENTRY = 'tauri-stub';

/** 桩入口的对外地址，用于注入到 index.html */
const MOCK_TAURI_SRC = `/mock/${MOCK_TAURI_ENTRY}.js`;

/**
 * 纯浏览器预览用的 Tauri IPC 桩（仅在 `--mode mock` 生效）。
 *
 * 页面依赖 @tauri-apps/api 的 invoke / listen，普通浏览器里拿不到这些外部对象，
 * 一进来就会抛异常白屏。这个插件在 mock 模式下：
 *
 *  1) 把 @tauri-apps/api/core 与 /event alias 到 mock/tauri-stub.ts
 *  2) 把桩作为独立入口注入 index.html，先于 main.ts 执行
 *
 * 默认 mode 下插件完全不介入：既不 alias、不注入，也不把桩加进入口，
 * 因此生产包里不会出现任何 mock 代码。
 */
export function viteMockTauri(): Plugin {
  let enabled = false;

  return {
    name: 'loemby:mock-tauri',
    enforce: 'pre',

    config(_config, env) {
      enabled = env.mode === 'mock';
      if (!enabled) {
        return undefined;
      }
      return {
        resolve: {
          alias: {
            '@tauri-apps/api/core': resolve(__dirname, 'mock/tauri-stub.ts'),
            '@tauri-apps/api/event': resolve(__dirname, 'mock/tauri-stub.ts'),
          },
        },
      };
    },

    transformIndexHtml(html) {
      if (!enabled) {
        return html;
      }
      return html.replace(
        '<script type="module" src="/src/main.ts"></script>',
        `<script type="module" src="${MOCK_TAURI_SRC}"></script>\n    <script type="module" src="/src/main.ts"></script>`,
      );
    },
  };
}
