-- SOURCE: https://github.com/kelciour/mpv-scripts/blob/master/sub-export.lua
-- COMMIT: 29 Aug 2018 5039d8b
--
-- Usage:
-- add bindings to input.conf:
-- key   script-message-to sub_export export-selected-subtitles
--
--  Note:
--     Requires FFmpeg in PATH environment variable or edit ffmpeg_path in the script options,
--     for example, by replacing "ffmpeg" with "C:\Programs\ffmpeg\bin\ffmpeg.exe"
--  Note:
--     The script support subtitles in srt, ass, and sup formats.
--  Note:
--     A small circle at the top-right corner is a sign that export is happenning now.
--  Note:
--     The exported subtitles will be automatically selected with visibility set to true.
--  Note:
--     It could take ~1-5 minutes to export subtitles.

local msg = require 'mp.msg'
local utils = require 'mp.utils'
local options = require "mp.options"

---- Script Options ----
local o = {
    ffmpeg_path = "ffmpeg",
    -- eng=English, chs=Chinese
    language = "eng",
}

options.read_options(o)
------------------------

local is_windows = package.config:sub(1, 1) == "\\" -- detect path separator, windows uses backslashes

-- 获取用户主目录
local user_home = os.getenv("HOME") or os.getenv("USERPROFILE") or "."

-- 全局变量，用于在 process 函数中引用
local subtitles_file = nil
local args = nil

-- 检测 ffmpeg 是否可用
local function check_ffmpeg()
    local test_args = is_windows and { 'powershell', '-NoProfile', '-Command', o.ffmpeg_path .. " -version" } 
                                 or { '/bin/bash', '-c', o.ffmpeg_path .. " -version" }
    local res = mp.command_native({
        name = "subprocess",
        playback_only = false,
        args = test_args,
        capture_stdout = true,
        capture_stderr = true
    })
    return res and res.status == 0
end

local function export_selected_subtitles()
    -- 检查 ffmpeg 是否存在
    if not check_ffmpeg() then
        if o.language == 'chs' then
            msg.info("错误: 未找到 ffmpeg，请确保已安装并在配置中指定正确的路径")
            mp.osd_message("错误: 未找到 ffmpeg，请检查配置", 3)
        else
            msg.info("Error: ffmpeg not found, please check your configuration")
            mp.osd_message("Error: ffmpeg not found, check configuration", 3)
        end
        return
    end

    local i = 0
    local tracks_count = mp.get_property_number("track-list/count")
    while i < tracks_count do
        local track_type = mp.get_property(string.format("track-list/%d/type", i))
        local track_index = mp.get_property_number(string.format("track-list/%d/ff-index", i))
        local track_selected = mp.get_property(string.format("track-list/%d/selected", i))
        local track_title = mp.get_property(string.format("track-list/%d/title", i))
        local track_lang = mp.get_property(string.format("track-list/%d/lang", i))
        local track_external = mp.get_property(string.format("track-list/%d/external", i))
        local track_codec = mp.get_property(string.format("track-list/%d/codec", i))
        local path = mp.get_property('path')
        
        if not path then
            return
        end

        local dir, filename = utils.split_path(path)
        local fname = mp.get_property("filename/no-ext")
        
        -- 判断是否为网络流（无本地真实视频目录）
        local is_network_stream = not path:match("^%a:[\\/]") and not path:match("^/") and not path:match("^\\\\")
        if path:match("^https?://") or path:match("^magnet:") or path:match("^ytdl://") then
            is_network_stream = true
        end

        if is_network_stream then
            -- 如果是网络流，将目录重定向到用户主目录
            dir = user_home
            if not fname or fname == "" then
                fname = mp.get_property("media-title") or ("stream_" .. os.time())
            end
            -- 过滤掉文件名中可能非法的字符
            fname = fname:gsub('[%\\/:%*%?\"<>%|]', '_')
        end

        local index = string.format("0:%d", track_index)

        if track_type == "sub" and track_selected == "yes" then
            if track_external == "yes" then
                if o.language == 'chs' then
                    msg.info("错误:已选择外部字幕")
                    mp.osd_message("错误:已选择外部字幕", 2)
                else
                    msg.info("Error: external subtitles have been selected")
                    mp.osd_message("Error: external subtitles have been selected", 2)
                end
                return
            end

            local subtitles_ext = ".srt"
            if string.find(track_codec, "ass") ~= nil then
                subtitles_ext = ".ass"
            elseif string.find(track_codec, "pgs") ~= nil then
                subtitles_ext = ".sup"
            end

            if track_lang ~= nil then
                if track_title ~= nil then
                    subtitles_ext = "." .. track_title .. "." .. track_lang .. subtitles_ext
                else
                    subtitles_ext = "." .. track_lang .. subtitles_ext
                end
            end

            subtitles_file = utils.join_path(dir, fname .. subtitles_ext)

            if o.language == 'chs' then
                msg.info("正在导出当前字幕")
                mp.osd_message("正在导出当前字幕")
            else
                msg.info("Exporting selected subtitles")
                mp.osd_message("Exporting selected subtitles")
            end

            local cmd = string.format("%s -y -hide_banner -loglevel error -i '%s' -map '%s' -vn -an -c:s copy '%s'",
                o.ffmpeg_path, path, index, subtitles_file)
            local windows_args = { 'powershell', '-NoProfile', '-Command', cmd }
            local unix_args = { '/bin/bash', '-c', cmd }
            args = is_windows and windows_args or unix_args

            mp.add_timeout(mp.get_property_number("osd-duration") * 0.001, process)

            break
        end

        i = i + 1
    end
end

function process()
    local screenx, screeny, aspect = mp.get_osd_size()

    mp.set_osd_ass(screenx, screeny, "{\\an7}● Exporting selected subtitles")
    local res = mp.command_native({ name = "subprocess", capture_stdout = true, playback_only = false, args = args })
    mp.set_osd_ass(screenx, screeny, "")
    
    if res.status == 0 then
        if o.language == 'chs' then
            msg.info("当前字幕已导出至: " .. subtitles_file)
            mp.osd_message("字幕已导出: " .. subtitles_file, 4)
        else
            msg.info("Finished exporting subtitles to: " .. subtitles_file)
            mp.osd_message("Subtitles exported: " .. subtitles_file, 4)
        end
        mp.commandv("sub-add", subtitles_file)
        mp.set_property("sub-visibility", "yes")
    else
        if o.language == 'chs' then
            msg.info("当前字幕导出失败")
            mp.osd_message("当前字幕导出失败, 查看控制台获取更多信息.", 3)
        else
            msg.info("Failed to export subtitles")
            mp.osd_message("Failed to export subtitles, check console for more info.", 3)
        end
    end
end

mp.register_script_message("export-selected-subtitles", export_selected_subtitles)
