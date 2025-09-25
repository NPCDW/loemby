local mp = require 'mp'
local utils = require 'mp.utils'

-- 完整的ASS样式定义，添加\pos(x,y)控制位置
-- 这里保持\an9(右上角对齐)，但用\pos从顶部向下偏移20像素
local ass_style = "{\\an9\\3c&HA066FD&}"
function update_speed()
    local cache_speed = mp.get_property_number("cache-speed", 0) / 1024 /1024  -- MB/s
    
    -- 自定义显示格式
    local speed_text = string.format("%.1f MB/s", cache_speed)
    -- 设置OSD显示
    mp.set_osd_ass(0, 0, ass_style .. speed_text)
    
    -- 每秒调用一次
    mp.add_timeout(1, update_speed)
end

-- 初始调用
update_speed()