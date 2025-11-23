use crate::service::emby_http_svc::MediaSource;
use std::net::Ipv4Addr;

pub fn get_display_title_from_media_sources(media_source: &MediaSource) -> String {
    let video_stream = media_source
        .media_streams
        .iter()
        .find(|stream| stream.type_ == "Video");

    match video_stream {
        Some(stream) => format!("{} / {}", media_source.name.clone(), stream.display_title.clone().unwrap_or("".to_string())),
        None => media_source.name.clone(),
    }
}

pub fn get_resolution_from_media_sources(media_source: &MediaSource) -> String {
    let video_stream = media_source
        .media_streams
        .iter()
        .find(|stream| stream.type_ == "Video");
    
    let mut media_streams_display_title = "".to_string();
    if let Some(video_stream) = video_stream {
        let resolution = get_resolution(video_stream.width, video_stream.height);
        if resolution != "Unknown" {
            return resolution;
        }
        media_streams_display_title = video_stream.display_title.clone().unwrap_or("".to_string()).to_lowercase();
    }

    let media_sources_name = format!("{} {}", media_streams_display_title, media_source.name.to_lowercase());
    if media_sources_name.contains("8k") || media_sources_name.contains("4320p") {
        return "8k".to_string()
    } else if media_sources_name.contains("4k") || media_sources_name.contains("2160p") {
        return "4k".to_string()
    } else if media_sources_name.contains("2k") || media_sources_name.contains("1440p") {
        return "2k".to_string()
    } else if media_sources_name.contains("1080p") {
        return "1080p".to_string()
    } else if media_sources_name.contains("720p") {
        return "720p".to_string()
    } else if media_sources_name.contains("480p") {
        return "480p".to_string()
    } else if media_sources_name.contains("360p") {
        return "360p".to_string()
    } else if media_sources_name.contains("240p") {
        return "240p".to_string()
    }
    return "Unknown".to_string();
}

pub fn get_resolution(width: Option<u32>, height: Option<u32>) -> String {
    match (width, height) {
        (Some(w), Some(h)) => {
            if w >= 7680 || h >= 4320 {
                "8K".to_string()
            } else if w >= 3840 || h >= 2160 {
                "4K".to_string()
            } else if w >= 2560 || h >= 1440 {
                "2K".to_string()
            } else if w >= 1920 || h >= 1080 {
                "1080p".to_string()
            } else if w >= 1280 || h >= 720 {
                "720p".to_string()
            } else if w >= 854 || h >= 480 {
                "480p".to_string()
            } else if w >= 640 || h >= 360 {
                "360p".to_string()
            } else {
                "240p".to_string()
            }
        }
        _ => "Unknown".to_string(),
    }
}

pub fn get_resolution_level_from_media_sources(media_source: &MediaSource) -> u32 {
    let video_stream = media_source
        .media_streams
        .iter()
        .find(|stream| stream.type_ == "Video");
    
    let mut media_streams_display_title = "".to_string();
    if let Some(video_stream) = video_stream {
        let level = get_resolution_level(video_stream.width, video_stream.height);
        if level != 0 {
            return level;
        }
        media_streams_display_title = video_stream.display_title.clone().unwrap_or("".to_string()).to_lowercase();
    }

    let media_sources_name = format!("{} {}", media_streams_display_title, media_source.name.to_lowercase());
    if media_sources_name.contains("8k") || media_sources_name.contains("4320p") {
        return 8
    } else if media_sources_name.contains("4k") || media_sources_name.contains("2160p") {
        return 7
    } else if media_sources_name.contains("2k") || media_sources_name.contains("1440p") {
        return 6
    } else if media_sources_name.contains("1080p") {
        return 5
    } else if media_sources_name.contains("720p") {
        return 4
    } else if media_sources_name.contains("480p") {
        return 3
    } else if media_sources_name.contains("360p") {
        return 2
    } else if media_sources_name.contains("240p") {
        return 1
    }
    return 0;
}

fn get_resolution_level(width: Option<u32>, height: Option<u32>) -> u32 {
    match (width, height) {
        (Some(w), Some(h)) => {
            if w >= 7680 || h >= 4320 {
                8
            } else if w >= 3840 || h >= 2160 {
                7
            } else if w >= 2560 || h >= 1440 {
                6
            } else if w >= 1920 || h >= 1080 {
                5
            } else if w >= 1280 || h >= 720 {
                4
            } else if w >= 854 || h >= 480 {
                3
            } else if w >= 640 || h >= 360 {
                2
            } else {
                1
            }
        }
        _ => 0,
    }
}

// 格式化字节大小
pub fn format_bytes(size: u64) -> String {
    if size == 0 {
        return "0 KB".to_string();
    }
    
    let units = ["KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    
    for (index, unit) in units.iter().enumerate() {
        size /= 1024.0;
        if size < 1024.0 || index == units.len() - 1 {
            return format!("{:.2} {}", size, unit);
        }
    }
    
    format!("{:.2} {}", size, units[units.len() - 1])
}

// 格式化 Mbps
pub fn format_mbps(size: u64) -> String {
    if size == 0 {
        return "0 Kbps".to_string();
    }
    
    let units = ["Kbps", "Mbps", "Gbps", "Tbps"];
    let mut size = size as f64;
    
    for (index, unit) in units.iter().enumerate() {
        size /= 1024.0;
        if size < 1024.0 || index == units.len() - 1 {
            return format!("{:.2} {}", size, unit);
        }
    }
    
    format!("{:.2} {}", size, units[units.len() - 1])
}

// 检查是否为内部 URL
pub fn is_internal_url(url_string: &str) -> bool {
    let url = match url::Url::parse(url_string) {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Invalid URL: {}", e);
            return false;
        }
    };
    
    let hostname = match url.host() {
        Some(host) => host,
        None => return false,
    };
    
    // 检查 localhost
    if hostname == url::Host::Domain("localhost") {
        return true;
    }
    
    // 检查 IPv6 环回地址
    if let url::Host::Ipv6(ipv6) = hostname {
        if ipv6.is_loopback() {
            return true;
        }
    }
    
    // 检查 IPv4 地址
    if let url::Host::Ipv4(ipv4) = hostname {
        return is_private_ipv4(ipv4);
    }
    
    // 检查域名
    if let url::Host::Domain(domain) = hostname {
        return is_likely_internal_domain(domain);
    }
    
    false
}

// 检查是否为私有 IPv4 地址
fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    let part1 = octets[0];
    let part2 = octets[1];
    
    // 10.0.0.0 - 10.255.255.255
    if part1 == 10 {
        return true;
    }
    
    // 172.16.0.0 - 172.31.255.255
    if part1 == 172 && part2 >= 16 && part2 <= 31 {
        return true;
    }
    
    // 192.168.0.0 - 192.168.255.255
    if part1 == 192 && part2 == 168 {
        return true;
    }
    
    // 127.0.0.0 - 127.255.255.255 (环回地址)
    if part1 == 127 {
        return true;
    }
    
    // 169.254.0.0 - 169.254.255.255 (链路本地地址)
    if part1 == 169 && part2 == 254 {
        return true;
    }
    
    false
}

// 检查是否为可能的内部域名
fn is_likely_internal_domain(hostname: &str) -> bool {
    let internal_tlds = [
        ".local",
        ".localhost",
        ".internal",
        ".intranet",
        ".lan",
        ".home",
        ".corp",
        ".office",
        ".localdomain",
        ".test",
    ];
    
    internal_tlds.iter().any(|tld| hostname.ends_with(tld))
}