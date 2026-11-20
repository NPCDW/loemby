
local mp = require "mp"
local msg = require "mp.msg"

-- 加载 LuaJIT FFI
local ffi_ok, ffi = pcall(require, "ffi")

if not ffi_ok then
    msg.error("当前 mpv 不支持 LuaJIT FFI，无法调用 Windows API")
    return
end

-- 声明 Windows API
ffi.cdef[[
    typedef void *mpv_ws_HANDLE;
    typedef int mpv_ws_BOOL;
    typedef unsigned long mpv_ws_DWORD;

    mpv_ws_HANDLE __stdcall GetCurrentProcess(void);

    mpv_ws_BOOL __stdcall K32EmptyWorkingSet(
        mpv_ws_HANDLE hProcess
    );

    mpv_ws_DWORD __stdcall GetLastError(void);
]]

-- 加载 Windows Kernel32
local ok, kernel32 = pcall(ffi.load, "kernel32")

if not ok then
    msg.error("无法加载 kernel32.dll: " .. tostring(kernel32))
    return
end

-- 清理当前 mpv 进程的工作集
local function empty_working_set()

    local call_ok, success, winerr = pcall(function()

        -- 获取当前 mpv 进程句柄
        local process = kernel32.GetCurrentProcess()

        -- 调用 Windows API
        local result = kernel32.K32EmptyWorkingSet(process)

        if result == 0 then
            return false, tonumber(kernel32.GetLastError())
        end

        return true, nil
    end)

    if not call_ok then
        msg.error("EmptyWorkingSet 调用异常: " .. tostring(success))

    elseif not success then
        msg.warn("EmptyWorkingSet 调用失败，错误代码: " .. tostring(winerr))

    else
        msg.info("已清理 mpv Working Set")
    end
end

-- 在上一个文件完全卸载后、下一个文件开始加载前执行
mp.add_hook("on_after_end_file", 50, function()
    empty_working_set()
end)

msg.info("EmptyWorkingSet 脚本加载成功")
