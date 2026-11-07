package com.loemby.mpv

import android.app.Activity
import android.content.Intent
import android.os.Handler
import android.os.Looper
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

@InvokeArg
class NativeOptions {
    var title: String? = null
    var startPaused: Boolean = true
    var configPath: String? = null
    // Rust Vec<(String, String)> is a JSON array of two-element arrays, not
    // Kotlin Pair objects. Tauri 2.12 uses Jackson; Java arrays also work with Gson.
    var mpvOptions: Array<Array<String?>?>? = emptyArray()

    fun nativeMpvOptions(): Array<String> {
        configPath?.let { path ->
            require(path.isNotBlank() && !path.contains('\u0000')) { "mpv 配置路径为空或包含 NUL" }
        }
        val pairs = requireNotNull(mpvOptions) { "mpvOptions 必须是选项数组" }
        return pairs.flatMapIndexed { index, pair ->
            require(pair != null && pair.size == 2) { "mpvOptions[$index] 必须是 [name, value]" }
            val name = requireNotNull(pair[0]) { "mpvOptions[$index] 的名称不能为空" }
            val value = requireNotNull(pair[1]) { "mpvOptions[$index] 的值不能是 null" }
            require(name.isNotEmpty() && !name.contains('\u0000') && !value.contains('\u0000')) {
                "mpvOptions[$index] 名称为空或包含 NUL"
            }
            listOf(name, value)
        }.toTypedArray()
    }
}

@InvokeArg
class PrepareArgs {
    lateinit var sessionId: String
    lateinit var socketPath: String
    var options: NativeOptions = NativeOptions()
}

@InvokeArg
class CloseArgs { lateinit var sessionId: String }

@TauriPlugin
class MpvPlugin(private val host: Activity) : Plugin(host) {
    @Command
    fun prepare(invoke: Invoke) {
        val args = try { invoke.parseArgs(PrepareArgs::class.java) } catch (error: Exception) {
            invoke.reject(error.message); return
        }
        host.runOnUiThread {
            if (args.sessionId.isBlank() || args.socketPath.isBlank()) {
                invoke.reject("播放会话或 socket 路径为空")
                return@runOnUiThread
            }
            if (MpvSessions.active != null) {
                invoke.reject("已有原生播放会话")
                return@runOnUiThread
            }
            val session = NativeSession(args, invoke)
            MpvSessions.active = session
            try {
                host.startActivity(Intent(host, MpvPlayerActivity::class.java).putExtra("sessionId", args.sessionId))
                Handler(Looper.getMainLooper()).postDelayed({
                    if (MpvSessions.active === session && session.pending != null) {
                        session.reject("等待播放 Surface 超时")
                        session.player?.closePlayer()
                        if (session.player == null) MpvSessions.active = null
                    }
                }, 25_000)
            } catch (error: Exception) {
                session.reject(error.message ?: "无法打开播放器")
                MpvSessions.active = null
            }
        }
    }

    @Command
    fun close(invoke: Invoke) {
        val args = try { invoke.parseArgs(CloseArgs::class.java) } catch (error: Exception) {
            invoke.reject(error.message); return
        }
        host.runOnUiThread {
            val session = MpvSessions.active
            if (session == null || session.args.sessionId != args.sessionId) {
                invoke.resolve()
            } else {
                session.closeReplies.add(invoke)
                session.reject("播放准备已取消")
                session.player?.closePlayer()
                if (session.player == null) MpvSessions.release(session)
            }
        }
    }
}

internal class NativeSession(val args: PrepareArgs, var pending: Invoke?) {
    var player: MpvPlayerActivity? = null
    val closeReplies = mutableListOf<Invoke>()

    fun ready() {
        pending?.resolve(JSObject().apply {
            put("sessionId", args.sessionId)
            put("socketPath", args.socketPath)
        })
        pending = null
    }

    fun reject(message: String) {
        pending?.reject(message)
        pending = null
    }
}

internal object MpvSessions {
    var active: NativeSession? = null

    fun release(session: NativeSession) {
        if (active !== session) return
        session.reject("播放器已关闭")
        active = null
        session.closeReplies.forEach { it.resolve() }
        session.closeReplies.clear()
        session.player = null
    }
}
