# 关于反代

## 追踪与应用数据代理

在「设置 → 追踪」中，Trakt、Simkl、YamTrack 分别提供独立的正向代理和反向代理选项；在「设置 → Emby图标库」中，「应用数据代理」也提供这两个选项。反向代理默认不启用，旧配置的正向代理及其继承行为不变，无需迁移数据库。

先在「反代服务器」中添加地址，再在相应服务中选择。这里的反代沿用下文第二种模式的完整目标 URL 网关协议，并非单纯替换 API 域名：

```text
反代地址：https://proxy.example.org/
原始目标：https://api.trakt.tv/scrobble/start
请求目标：https://proxy.example.org/https://api.trakt.tv/scrobble/start
```

两种代理可同时使用，路径为「应用 → 正向代理 → 反代服务器 → 原始目标」。更改追踪或应用数据代理后，后续请求使用新的配置；已经提示可安装的更新也会在安装前重新读取代理配置。

反代需要正确传递请求方法、请求体和 `Authorization`、API Key 等请求头，并按下文的 `proxy_redirect` 示例改写 `Location`。追踪的 Token 交换请求会访问 `token-exchange.i101.workers.dev`，不能仅允许 `api.trakt.tv` 或 `api.simkl.com`；更新还需要允许清单中的安装包地址及 GitHub 重定向目标。

**请只选择可信的反代服务器，并优先使用 HTTPS。** 反代会接触追踪令牌、API Key、Webhook 地址及请求内容。YamTrack 和图标库的自建/LAN 地址也会按所选代理转发，反代必须能访问这些目标。

Trakt/Simkl 的浏览器授权页面仍通过系统浏览器打开，使用浏览器自己的网络/代理设置，不会自动套用应用代理。应用收到本地回调后的 Token 交换、Trakt Token 刷新和账户查询会使用相应追踪代理；OAuth 的 `127.0.0.1` 回调地址和请求体中的 `redirect_uri` 不改写。

## 应用 HTTP 请求覆盖检查

「应用数据代理」不是所有 HTTP 请求的统一策略：Emby 使用服务器/线路的浏览或播放设置，追踪服务各自使用独立设置。以下为应用自有代码的检查结果：

| 请求类别 | 正向代理 | 反向代理 | 说明 |
| --- | --- | --- | --- |
| Trakt Token 交换/刷新、用户查询、开始/停止同步 | Trakt 设置 | Trakt 设置 | 交换网关和 API 均覆盖 |
| Simkl Token 交换、用户查询、开始/暂停/停止同步 | Simkl 设置 | Simkl 设置 | 暂停与停止使用同一路由 |
| YamTrack Webhook | YamTrack 设置 | YamTrack 设置 | 原始自定义地址不变 |
| 图标库 JSON、应用图标图片 | 应用数据设置 | 应用数据设置 | 缓存命中时不发起远程请求 |
| 更新清单检查、安装包下载 | 应用数据设置 | 应用数据设置 | 下载前读取当前配置；保留签名校验 |
| Emby API（含登录、播放信息和进度上报）、海报 | 浏览代理 | 服务器/线路反代 | 原有覆盖不变 |
| Emby 相对媒体地址、字幕转发及预下载 | 播放代理 | 服务器/线路反代 | 原有覆盖不变；媒体例外见下文 |
| 正向/反代连通性检测 | 使用被检测的正向代理，或不指定正向代理 | 使用被检测的反代，或不使用反代 | 有意独立于应用数据设置 |
| 应用本地 Axum、OAuth 回调、Tauri IPC | 不套用远程服务设置 | 不套用远程服务设置 | 本地通信不是待代理的外部 API |

### 本次未修改的遗漏与边界

- **媒体绝对直链可能绕过反代**：`src-tauri/src/service/axum_svc.rs` 的 `stream()` 仅给相对地址拼接 `reverse_base_url`；绝对远程直链仍使用播放正向代理，但不自动添加反代。
- **媒体手动重定向处理不完整**：同一函数把 `Location` 直接当作下一次请求地址，相对重定向可能失败，未改写的绝对重定向可能离开反代。需要反代服务端正确改写 `Location`。
- **HLS 相对链接未封装为本地转发地址**：现有解析只替换绝对 HTTP(S) URL；相对分片、相对播放列表、协议相对 URL 和相对密钥 URI 可能解析错误或绕过应用转发。已有反代链接还可能被重复加前缀。按本次范围，不修改媒体播放逻辑。
- **自动重定向不保证客户端始终经过反代**：API、图片、字幕及 updater 客户端自动跟随重定向，依赖服务端改写 `Location`；本次新增初始请求路由不代表已补齐每一次重定向。
- **「不使用代理」仍可能继承系统/环境代理**：当前 HTTP 客户端不主动禁用系统代理，`None` 表示未指定应用内正向代理。本次不改变这一全局语义。
- **系统浏览器和外部程序有独立的网络行为**：官网授权、通知中的服务链接及文档链接由系统浏览器处理；常规 mpv/FFmpeg 播放使用本地转发 URL，但用户自定义播放地址、二进制或脚本可能自行联网，不受应用设置保证。
- **构建/CI 的网络请求不使用应用设置**：依赖安装、GitHub API/安装包下载、构建时拉取 uosc、仓库同步使用各自工具的环境配置；外部脚本及未随仓库提供的运行时代码无法在此检查中完整覆盖。
- 代理服务器被删除/配置 ID 不存在时，现有解析会回退为无显式代理；HTTP 连接池缓存不会因证书校验开关变化而重建。这些现有行为也未在本次调整。

## Emby 反代说明

反代首先有一个反代的服务器，然后客户端通过反代服务器访问 Emby 服务器，反代服务器将请求转发给 Emby 服务器，然后将 Emby 服务器的响应转发给客户端。

如果只配置了反代服务器端，不配置客户端，对于绝大多数的 Emby 服务器有效，绝大多数的 Emby 服务器播放都是给一个 mkv 或 mp4 播放地址，但是对一些视频播放使用 m3u8 协议的 Emby 服务器不适用。

所以本客户端需要解决的就是 m3u8 协议的 Emby 服务器播放的问题，已解决，方案就是当获取到的时 m3u8 时，把文件解析，并将里面的地址加上反代服务器的地址，再封装回 m3u8 文件，然后再返回给播放器。

需要注意的是，反代服务强烈推荐使用 https ，但是使用 https 会导致别人都能查到你配置了那些 ssl 证书，从而获取到你的反代域名，这个网站 [crt.sh](https://crt.sh/) 就可以查到，
所以建议使用通配符证书，然后配置一个随机字符串的域名，例如：申请一个 `*.emby.example.com` 的通配符证书，然后配置一个 `c7f47117-3c13-436e-8176-b6c8d074d5ae.emby.example.com` 的域名，这样别人就查不到你的反代域名了。

反代的两种模式:
1. 只对一个 Emby 服务器反代，本客户端没有适配这种配置的 m3u8 播放格式，但是对其他情况都有效，需要注意的是，这种反代在本客户端只需要配置服务器地址，不要配置反代服务器地址，nginx 配置示例如下：
```nginx
server {
    listen 443 ssl http2;
    # 你的域名
    server_name p.example.com;
    # 你的证书
    ssl_certificate /root/cert/example.com.cer;
    ssl_certificate_key /root/cert/example.com.key;

    client_max_body_size 20M;
    add_header X-Frame-Options "SAMEORIGIN";
    add_header X-XSS-Protection "1; mode=block";
    add_header X-Content-Type-Options "nosniff";

    location / {
        # 你需要反代的emby服务器域名
        proxy_pass https://emby.example.com;
        # 你需要反代的emby推流地址
        proxy_redirect https://stream1.example.com/ https://p.example.com/s1/;
        proxy_redirect https://stream2.example.com/ https://p.example.com/s2/;
        # 你需要反代的emby服务器主页
        proxy_set_header Referer "https://emby.example.com/web/index.html"; 
        proxy_set_header Upgrade $http_upgrade; 
        proxy_set_header Connection $connection_upgrade;
        proxy_set_header Host $proxy_host; 
        proxy_ssl_server_name on; 
        proxy_http_version 1.1;
    }

    location /s1 {
        rewrite ^/s1(/.*)$ $1 break;
        # 你需要反代的emby推流地址
        proxy_pass https://stream1.example.com/;
        proxy_set_header Referer "https://emby.example.com/web/index.html";
        proxy_set_header Host $proxy_host;
        proxy_ssl_server_name on;
        proxy_buffering off;
    }
    location /s2 {
        rewrite ^/s2(/.*)$ $1 break;
        # 你需要反代的emby推流地址
        proxy_pass https://stream2.example.com/;
        proxy_set_header Referer "https://emby.example.com/web/index.html";
        proxy_set_header Host $proxy_host;
        proxy_ssl_server_name on;
        proxy_buffering off;
    }
}
```
2. 对多个 Emby 服务器反代，本客户端对这种有良好的适配，本客户端需要配置反代服务器地址，而 Emby 服务器地址只需要配置 Emby 提供者提供的即可，nginx 配置示例如下：
```nginx
server {
    listen       80;
    listen       [::]:80;
    server_name 83d834712f8f.emby.example.com v6-83d834712f8f.emby.example.com;

    rewrite ^(.*) https://$server_name$1 permanent; 
}

server {
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name 83d834712f8f.emby.example.com v6-83d834712f8f.emby.example.com;

    ssl_certificate   /etc/ssl/nginx/all.emby/all.emby.example.com.pem;
    ssl_certificate_key  /etc/ssl/nginx/all.emby/all.emby.example.com.key;

    ssl_session_timeout 5m;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES128-GCM-SHA256:HIGH:!aNULL:!MD5:!RC4:!DHE;
    ssl_prefer_server_ciphers on;

    ssl_session_cache shared:SSL:1m;

    fastcgi_param  HTTPS        on;
    fastcgi_param  HTTP_SCHEME     https;

    ignore_invalid_headers off;

    client_max_body_size 100m;

    proxy_buffering off;
    proxy_request_buffering off;
    # 开启 DNS 解析器以支持动态域名转发（使用 Google 和 Cloudflare Public DNS）
    # valid=30s 表示 DNS 缓存时间，避免频繁请求，为防止 v4 和 v6 去向漂移，可以设置 ipv6=off 以及只设置一个 DNS
    resolver 8.8.8.8 1.1.1.1 valid=30s;
    resolver_timeout 5s;
    
    # 关键点 1：禁止 Nginx 自动压缩双斜杠 //
    merge_slashes off;
    
    # 使用原始未解码的 $request_uri 进行正则匹配与提取
    location ~* ^/(https?):/+(.+) {
        # 利用正则从原始请求 $request_uri 中精准提取 scheme、host 和原始带编码的 path
        if ($request_uri ~* ^/(https?):/+([^/]+)(/.*)?$) {
            set $target_scheme $1;
            set $target_host $2;
            set $target_path $3;
        }

        if ($target_path = "") {
            set $target_path "/";
        }

        # 关键：直接改写 URI，避免 proxy_pass 变量拼接导致二次编码
        rewrite ^/(https?):/+([^/]+)(/.*)?$ $3 break;

        # ----------------- 关键修改：配置 proxy_redirect -----------------
        # 1. 重写绝对路径重定向 (如 Location: http(s)://ccccc.com/path)
        # 将其替换为: https://http-proxy.aaaa.dev/http(s)://ccccc.com/path
        proxy_redirect ~*^(https?://.+) https://$http_host/$1;

        # 2. 重写相对路径重定向 (如 Location: /path/to/page)
        # 将其替换为: https://http-proxy.aaaa.dev/https://bbbbb.com/path/to/page
        proxy_redirect ~^/(.*)$ https://$http_host/$target_scheme://$target_host/$1;
        # -----------------------------------------------------------------

        # 不要开启 error_page 拦截，让 301/302 直接透传（但 Location 头已被上面重写）
        proxy_intercept_errors off;

        # 清除反代特征头
        proxy_set_header X-Forwarded-For "";
        proxy_set_header X-Real-IP "";
        proxy_set_header X-Forwarded-Proto "";
        proxy_set_header X-Forwarded-Host "";
        proxy_set_header X-Forwarded-Port "";

        # 伪造 Host 和 基础头
        proxy_set_header Host $target_host;

        # 协议与 SNI
        proxy_http_version 1.1;
        proxy_ssl_server_name on;
        proxy_ssl_name $target_host;
        
        proxy_ssl_session_reuse on;
        # proxy_bind 网卡真实的地址; # 出口IP

        # 注意：这里 proxy_pass 不带 URI，URI 由 rewrite 后的 $uri 决定
        proxy_pass $target_scheme://$target_host;
    }
}
```