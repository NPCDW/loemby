package com.loemby.mpv

import android.app.Activity
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.graphics.Color
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.graphics.SurfaceTexture
import android.view.Surface
import android.view.TextureView
import android.view.Gravity
import android.view.WindowManager
import android.widget.Button
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.SeekBar
import android.widget.TextView
import android.widget.Toast
import java.util.concurrent.Executors

class MpvPlayerActivity : Activity(), TextureView.SurfaceTextureListener {
    private val worker = Executors.newSingleThreadExecutor()
    private val ui = Handler(Looper.getMainLooper())
    private var session: NativeSession? = null
    private var nativeHandle = 0L
    private var closing = false
    private var initializing = false
    private var playbackSurface: Surface? = null
    private var playbackTexture: SurfaceTexture? = null
    private var duration = 0.0
    private var dragging = false
    private lateinit var title: TextView
    private lateinit var time: TextView
    private lateinit var seek: SeekBar
    private lateinit var pause: Button
    private lateinit var audio: AudioManager
    private var focusRequest: AudioFocusRequest? = null
    private var ready = false
    private var preparedToken = ""
    private var switchingFrom = ""
    private var holdingFocus = false
    private lateinit var previous: Button
    private lateinit var next: Button
    private val focusListener = AudioManager.OnAudioFocusChangeListener { change ->
        if (change < 0) ui.post { holdingFocus = false; send("set", "pause", "yes") }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val active = MpvSessions.active
        if (active == null || active.args.sessionId != intent.getStringExtra("sessionId")) {
            finish(); return
        }
        session = active
        active.player = this
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        audio = getSystemService(AUDIO_SERVICE) as AudioManager
        val root = FrameLayout(this).apply { setBackgroundColor(Color.BLACK) }
        val surface = TextureView(this)
        root.addView(surface, FrameLayout.LayoutParams(-1, -1))
        surface.surfaceTextureListener = this
        val controls = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(16, 12, 16, 12)
            setBackgroundColor(0xb0000000.toInt())
        }
        title = TextView(this).apply { setTextColor(Color.WHITE); text = active.args.options.title ?: "Loemby" }
        time = TextView(this).apply { setTextColor(Color.WHITE); text = "正在准备播放器…" }
        seek = SeekBar(this).apply {
            max = 1000
            contentDescription = "播放进度"
            setOnSeekBarChangeListener(object : SeekBar.OnSeekBarChangeListener {
                override fun onStartTrackingTouch(bar: SeekBar) { dragging = true }
                override fun onProgressChanged(bar: SeekBar, value: Int, fromUser: Boolean) {}
                override fun onStopTrackingTouch(bar: SeekBar) {
                    dragging = false
                    if (duration > 0) send("seek", (bar.progress * duration / 1000).toString(), "absolute")
                }
            })
        }
        controls.addView(title); controls.addView(time); controls.addView(seek)
        val buttons = LinearLayout(this).apply { gravity = Gravity.CENTER }
        fun button(label: String, action: () -> Unit): Button = Button(this).apply {
            text = label; setOnClickListener { action() }
            buttons.addView(this, LinearLayout.LayoutParams(0, -2, 1f))
        }
        previous = button("上一集") { switchItem("playlist-prev") }
        pause = button("播放/暂停") { togglePause() }
        next = button("下一集") { switchItem("playlist-next") }
        previous.isEnabled = false; next.isEnabled = false; pause.isEnabled = false; seek.isEnabled = false
        button("退出") { closePlayer() }
        controls.addView(buttons)
        root.addView(controls, FrameLayout.LayoutParams(-1, -2, Gravity.BOTTOM))
        setContentView(root)
    }

    override fun onSurfaceTextureAvailable(texture: SurfaceTexture, width: Int, height: Int) {
        val active = session ?: return
        if (closing || initializing || nativeHandle != 0L) return
        initializing = true
        val surface = Surface(texture)
        playbackTexture = texture
        playbackSurface = surface
        worker.execute {
            try {
                MpvNative.load()
                val handle = MpvNative.create(
                    applicationContext, surface, active.args.socketPath,
                    active.args.options.startPaused, active.args.options.configPath,
                    active.args.options.nativeMpvOptions(),
                )
                nativeHandle = handle
                ui.post {
                    initializing = false
                    if (!closing) {
                        if (!requestAudioFocus()) {
                            active.reject("无法取得音频焦点")
                            closePlayer()
                        } else {
                            active.ready()
                            ui.post(update)
                        }
                    }
                }
            } catch (error: Throwable) {
                ui.post {
                    initializing = false
                    active.reject(error.message ?: "无法加载 Android mpv 原生库")
                    Toast.makeText(this, "播放器初始化失败：${error.message}", Toast.LENGTH_LONG).show()
                    closePlayer()
                }
            }
        }
    }

    override fun onSurfaceTextureSizeChanged(texture: SurfaceTexture, width: Int, height: Int) {
        if (closing) return
        worker.execute { if (nativeHandle != 0L) MpvNative.resize(nativeHandle, width, height) }
    }

    override fun onSurfaceTextureDestroyed(texture: SurfaceTexture): Boolean {
        closePlayer()
        // Retain the buffer producer until the worker has fully destroyed mpv's VO.
        return false
    }

    override fun onSurfaceTextureUpdated(texture: SurfaceTexture) {}

    private fun send(vararg args: String) {
        if (closing) return
        worker.execute {
            if (nativeHandle == 0L) return@execute
            try { MpvNative.command(nativeHandle, arrayOf(*args)) }
            catch (error: Exception) { ui.post { Toast.makeText(this, error.message, Toast.LENGTH_SHORT).show() } }
        }
    }

    private val update = object : Runnable {
        override fun run() {
            if (closing) return
            worker.execute {
                if (nativeHandle == 0L) return@execute
                val values = MpvNative.snapshot(nativeHandle)
                ui.post {
                    if (closing) return@post
                    if (values[4] == "yes") { closePlayer(); return@post }
                    val position = values[1].toDoubleOrNull() ?: 0.0
                    duration = values[2].toDoubleOrNull() ?: 0.0
                    title.text = values[0]
                    time.text = "${formatTime(position)} / ${formatTime(duration)}"
                    if (!dragging && duration > 0) seek.progress = (position * 1000 / duration).toInt().coerceIn(0, 1000)
                    val token = values[5]
                    val nowReady = token.isNotBlank() && token != "null" && token == values[6] && token != switchingFrom
                    if (nowReady && token != preparedToken) {
                        preparedToken = token
                        if (holdingFocus) send("set", "pause", "no")
                    }
                    ready = nowReady
                    val playlistPosition = values[7].toIntOrNull() ?: 0
                    val playlistCount = values[8].toIntOrNull() ?: 0
                    previous.isEnabled = ready && playlistPosition > 0
                    next.isEnabled = ready && playlistPosition + 1 < playlistCount
                    pause.isEnabled = ready; seek.isEnabled = ready && duration > 0
                    pause.text = if (values[3] == "yes") "播放" else "暂停"
                    ui.postDelayed(this, 500)
                }
            }
        }
    }

    fun closePlayer() {
        if (closing) return
        closing = true
        ui.removeCallbacks(update)
        val active = session
        active?.reject("播放器已关闭")
        worker.execute {
            try {
                if (nativeHandle != 0L) MpvNative.destroy(nativeHandle)
            } finally {
                nativeHandle = 0L
                playbackSurface?.release()
                playbackSurface = null
                playbackTexture?.release()
                playbackTexture = null
                ui.post {
                    abandonAudioFocus()
                    window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
                    if (active != null) MpvSessions.release(active)
                    if (!isFinishing) finish()
                    worker.shutdown()
                }
            }
        }
    }

    @Deprecated("Deprecated in Android")
    override fun onBackPressed() { closePlayer() }

    override fun onStop() { super.onStop(); closePlayer() }
    override fun onDestroy() { closePlayer(); super.onDestroy() }

    private fun switchItem(command: String) {
        if (!ready) return
        switchingFrom = preparedToken
        ready = false
        previous.isEnabled = false; next.isEnabled = false; pause.isEnabled = false; seek.isEnabled = false
        send(command, "weak")
    }

    private fun togglePause() {
        if (pause.text == "播放") {
            if (requestAudioFocus()) send("set", "pause", "no")
        } else {
            send("set", "pause", "yes")
            abandonAudioFocus()
        }
    }

    private fun requestAudioFocus(): Boolean {
        holdingFocus = if (android.os.Build.VERSION.SDK_INT >= 26) {
            focusRequest = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
                .setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_MOVIE).build())
                .setOnAudioFocusChangeListener(focusListener).build()
            audio.requestAudioFocus(focusRequest!!) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED
        } else {
            @Suppress("DEPRECATION")
            audio.requestAudioFocus(focusListener, AudioManager.STREAM_MUSIC, AudioManager.AUDIOFOCUS_GAIN) == AudioManager.AUDIOFOCUS_REQUEST_GRANTED
        }
        return holdingFocus
    }

    private fun abandonAudioFocus() {
        holdingFocus = false
        if (!::audio.isInitialized) return
        if (android.os.Build.VERSION.SDK_INT >= 26) focusRequest?.let { audio.abandonAudioFocusRequest(it) }
        else { @Suppress("DEPRECATION") audio.abandonAudioFocus(focusListener) }
    }

    private fun formatTime(seconds: Double): String {
        val value = seconds.toLong().coerceAtLeast(0)
        return "%d:%02d:%02d".format(value / 3600, value / 60 % 60, value % 60)
    }
}
