local mp = require 'mp'
local options = require 'mp.options'

-- 属性，默认启用，ASS样式定义，保持\an9(右上角对齐)
local o = {
    enabled = true,
    ass_style = "{\\an9\\3c&HA066FD&}"
}

options.read_options(o, "cache_speed")

function update_speed()
    local cache_speed = mp.get_property_number("cache-speed", 0) / 1024 /1024  -- MB/s
    
    -- 自定义显示格式
    local speed_text = string.format("%.1f MB/s", cache_speed)
    -- 设置OSD显示
    mp.set_osd_ass(0, 0, o.ass_style .. speed_text)
    
    -- 每秒调用一次
    mp.add_timeout(1, update_speed)
end

-- 初始调用
if o.enabled then
    update_speed()
end