# 关于反代

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
    listen 443 ssl;
    listen [::]:443 ssl;
    server_name 83d834712f8f.emby.example.com;

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
    # valid=30s 表示 DNS 缓存时间，避免频繁请求
    resolver 8.8.8.8 1.1.1.1 valid=30s;
    resolver_timeout 5s;
    
    # 关键点 1：禁止 Nginx 自动压缩双斜杠 //
    merge_slashes off;
    
    location ~* ^/(https?):/+(.+) {
        set $target_scheme $1;
        set $target_rest $2;

        if ($target_rest ~* "^([^/]+)(/.*)?$") {
            set $target_host $1;
            set $target_path $2;
        }

        if ($target_path = "") {
            set $target_path "/";
        }

        # ----------------- 关键修改：配置 proxy_redirect -----------------
        # 1. 重写绝对路径重定向 (如 Location: http(s)://ccccc.com/path)
        # 将其替换为: https://http-proxy.aaaa.dev/http(s)://ccccc.com/path
        proxy_redirect ~*^(https?://.+) https://83d834712f8f.emby.example.com/$1;

        # 2. 重写相对路径重定向 (如 Location: /path/to/page)
        # 将其替换为: https://http-proxy.aaaa.dev/https://bbbbb.com/path/to/page
        proxy_redirect ~^/(.*)$ https://83d834712f8f.emby.example.com/$target_scheme://$target_host/$1;
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

        proxy_pass $target_scheme://$target_host$target_path$is_args$args;
    }
}
```