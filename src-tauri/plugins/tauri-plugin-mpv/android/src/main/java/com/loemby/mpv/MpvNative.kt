package com.loemby.mpv

import android.content.Context
import android.view.Surface

object MpvNative {
    @Synchronized
    fun load() {
        if (loaded) return
        System.loadLibrary("mpv")
        System.loadLibrary("loemby_mpv_jni")
        loaded = true
    }

    private var loaded = false
    external fun create(
        context: Context,
        surface: Surface,
        socket: String,
        paused: Boolean,
        configPath: String?,
        mpvOptions: Array<String>,
    ): Long
    external fun destroy(handle: Long)
    external fun command(handle: Long, args: Array<String>)
    external fun snapshot(handle: Long): Array<String>
    external fun resize(handle: Long, width: Int, height: Int)
}
