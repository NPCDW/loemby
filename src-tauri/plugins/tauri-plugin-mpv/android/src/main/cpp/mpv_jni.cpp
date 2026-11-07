#include <jni.h>
#include <mpv/client.h>
extern "C" {
#include <libavcodec/jni.h>
}
#include <android/log.h>
#include <atomic>
#include <cstdint>
#include <memory>
#include <string>
#include <thread>
#include <codecvt>
#include <locale>

struct Player {
    mpv_handle *mpv = nullptr;
    jobject surface = nullptr;
    std::atomic<bool> stopping{false};
    std::atomic<bool> shutdown{false};
    std::thread events;
};

static void fail(JNIEnv *env, const std::string &message) {
    if (env->ExceptionCheck()) return;
    // ThrowNew expects modified UTF-8. Build a Java UTF-16 string instead so
    // supplementary characters in option names or config paths stay intact.
    std::u16string text;
    try { text = std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>().from_bytes(message); }
    catch (...) { text = u"libmpv error (invalid UTF-8 message)"; }
    jclass type = env->FindClass("java/lang/IllegalStateException");
    if (!type) return;
    jmethodID constructor = env->GetMethodID(type, "<init>", "(Ljava/lang/String;)V");
    if (!constructor) { env->DeleteLocalRef(type); return; }
    jstring value = env->NewString(reinterpret_cast<const jchar *>(text.data()), static_cast<jsize>(text.size()));
    if (value) {
        auto error = static_cast<jthrowable>(env->NewObject(type, constructor, value));
        if (error) { env->Throw(error); env->DeleteLocalRef(error); }
    }
    if (value) env->DeleteLocalRef(value);
    env->DeleteLocalRef(type);
}

static std::string utf8(JNIEnv *env, jstring value) {
    if (!value) { fail(env, "mpv 参数不能是 null"); return {}; }
    const jchar *text = env->GetStringChars(value, nullptr);
    if (!text) return {};
    std::u16string input(reinterpret_cast<const char16_t *>(text), env->GetStringLength(value));
    env->ReleaseStringChars(value, text);
    if (input.find(u'\0') != std::u16string::npos) {
        fail(env, "mpv 参数不能包含 NUL"); return {};
    }
    // codecvt implementations can accept a trailing unmatched high surrogate.
    for (size_t index = 0; index < input.size(); ++index) {
        const char16_t code = input[index];
        if (code >= 0xd800 && code <= 0xdbff) {
            if (++index >= input.size() || input[index] < 0xdc00 || input[index] > 0xdfff) {
                fail(env, "无效的 Unicode 参数"); return {};
            }
        } else if (code >= 0xdc00 && code <= 0xdfff) {
            fail(env, "无效的 Unicode 参数"); return {};
        }
    }
    try {
        return std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>().to_bytes(input);
    } catch (...) {
        fail(env, "无效的 Unicode 参数");
        return {};
    }
}

static void drain_logs(Player *player) {
    while (true) {
        mpv_event *event = mpv_wait_event(player->mpv, 0);
        if (event->event_id == MPV_EVENT_NONE) return;
        if (event->event_id == MPV_EVENT_LOG_MESSAGE) {
            auto *log = static_cast<mpv_event_log_message *>(event->data);
            __android_log_print(ANDROID_LOG_WARN, "LoembyMPV", "%s: %s", log->prefix, log->text);
        }
    }
}

static bool option(JNIEnv *env, Player *player, const char *name, const char *value) {
    if (env->ExceptionCheck()) return false;
    int error = mpv_set_option_string(player->mpv, name, value);
    if (error >= 0) return true;
    drain_logs(player);
    fail(env, std::string("mpv option ") + name + ": " + mpv_error_string(error));
    return false;
}

static bool user_options(JNIEnv *env, Player *player, jobjectArray options) {
    if (!options) { fail(env, "mpvOptions 不能是 null"); return false; }
    const jsize count = env->GetArrayLength(options);
    if (count % 2 != 0) { fail(env, "mpvOptions 必须按名称、值成对提供"); return false; }
    for (jsize index = 0; index < count; index += 2) {
        auto key = static_cast<jstring>(env->GetObjectArrayElement(options, index));
        if (env->ExceptionCheck()) return false;
        const std::string name = utf8(env, key);
        env->DeleteLocalRef(key);
        if (env->ExceptionCheck()) return false;
        if (name.empty()) { fail(env, "mpv 选项名称不能为空"); return false; }
        auto item = static_cast<jstring>(env->GetObjectArrayElement(options, index + 1));
        if (env->ExceptionCheck()) return false;
        const std::string value = utf8(env, item);
        env->DeleteLocalRef(item);
        if (env->ExceptionCheck()) return false;
        // No shell parsing or comma splitting. libmpv owns option value syntax.
        if (!option(env, player, name.c_str(), value.c_str())) return false;
    }
    return true;
}

static void destroy(JNIEnv *env, Player *player) {
    player->stopping = true;
    if (player->mpv) mpv_wakeup(player->mpv);
    if (player->events.joinable()) player->events.join();
    // terminate_destroy waits for VO teardown. Keep the Surface global ref alive until then.
    if (player->mpv) mpv_terminate_destroy(player->mpv);
    if (player->surface) env->DeleteGlobalRef(player->surface);
    delete player;
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_loemby_mpv_MpvNative_create(JNIEnv *env, jobject, jobject, jobject surface, jstring socket,
                                    jboolean paused, jstring config_path, jobjectArray options) {
    const std::string path = utf8(env, socket);
    if (env->ExceptionCheck()) return 0;
    const std::string config = config_path ? utf8(env, config_path) : std::string();
    if (env->ExceptionCheck()) return 0;
    if (path.empty() || (config_path && config.empty())) {
        fail(env, "mpv socket 或配置路径为空"); return 0;
    }
    JavaVM *vm = nullptr;
    if (env->GetJavaVM(&vm) != JNI_OK || !vm || av_jni_set_java_vm(vm, nullptr) < 0) {
        fail(env, "无法为 mpv 注册 JavaVM");
        return 0;
    }
    auto *player = new Player();
    player->surface = env->NewGlobalRef(surface);
    player->mpv = mpv_create();
    if (!player->surface || !player->mpv) {
        fail(env, "无法创建 libmpv 或 Surface 引用"); destroy(env, player); return 0;
    }
    // Subscribe before parsing: mpv can report individual config-line errors
    // while still returning success for the file as a whole.
    int result = mpv_request_log_messages(player->mpv, "warn");
    if (result < 0) {
        fail(env, std::string("无法启用 mpv 日志: ") + mpv_error_string(result));
        destroy(env, player); return 0;
    }
    // Native defaults only. In particular user config may replace hwdec.
    bool ok = option(env, player, "config", "no") && option(env, player, "load-scripts", "no") &&
        option(env, player, "hwdec", "mediacodec,mediacodec-copy");
    if (ok && config_path) {
        result = mpv_load_config_file(player->mpv, config.c_str());
        drain_logs(player);
        if (result < 0) {
            fail(env, std::string("mpv 配置加载失败 (") + config + "): " + mpv_error_string(result));
            destroy(env, player); return 0;
        }
    }
    if (ok) ok = user_options(env, player, options);
    // Override only native lifecycle/controls/surface invariants, after config
    // and shared application options. Explicit scripts are still supported.
    if (ok) {
        ok = option(env, player, "idle", "yes") &&
            option(env, player, "input-default-bindings", "no") &&
            option(env, player, "input-builtin-bindings", "no") &&
            option(env, player, "input-terminal", "no") && option(env, player, "input-vo-keyboard", "no") &&
            option(env, player, "input-cursor", "no") && option(env, player, "osc", "no") &&
            option(env, player, "vo", "gpu") && option(env, player, "gpu-context", "android") &&
            option(env, player, "opengl-es", "yes") &&
            option(env, player, "force-window", "yes") && option(env, player, "pause", paused ? "yes" : "no") &&
            option(env, player, "input-ipc-server", path.c_str());
    }
    int64_t wid = reinterpret_cast<intptr_t>(player->surface);
    result = ok ? mpv_set_option(player->mpv, "wid", MPV_FORMAT_INT64, &wid) : -1;
    if (ok && result < 0) fail(env, std::string("mpv option wid: ") + mpv_error_string(result));
    if (result >= 0) result = mpv_initialize(player->mpv);
    if (!ok || result < 0) {
        drain_logs(player);
        fail(env, std::string("libmpv 初始化失败: ") + mpv_error_string(result));
        destroy(env, player); return 0;
    }
    try {
        player->events = std::thread([player] {
            while (!player->stopping) {
                mpv_event *event = mpv_wait_event(player->mpv, -1);
                if (event->event_id == MPV_EVENT_SHUTDOWN) { player->shutdown = true; break; }
                if (event->event_id == MPV_EVENT_LOG_MESSAGE) {
                    auto *log = static_cast<mpv_event_log_message *>(event->data);
                    __android_log_print(ANDROID_LOG_WARN, "LoembyMPV", "%s: %s", log->prefix, log->text);
                }
            }
        });
    } catch (...) {
        fail(env, "无法创建 mpv 事件线程"); destroy(env, player); return 0;
    }
    return reinterpret_cast<intptr_t>(player);
}

extern "C" JNIEXPORT void JNICALL
Java_com_loemby_mpv_MpvNative_destroy(JNIEnv *env, jobject, jlong handle) {
    if (handle) destroy(env, reinterpret_cast<Player *>(handle));
}

extern "C" JNIEXPORT void JNICALL
Java_com_loemby_mpv_MpvNative_command(JNIEnv *env, jobject, jlong handle, jobjectArray arguments) {
    auto *player = reinterpret_cast<Player *>(handle);
    if (!player || player->shutdown) return;
    if (!arguments) { fail(env, "mpv 命令参数不能是 null"); return; }
    const jsize count = env->GetArrayLength(arguments);
    if (count > 32) { fail(env, "mpv 命令参数过多"); return; }
    std::string strings[32];
    const char *args[33] = {};
    for (jsize index = 0; index < count; ++index) {
        auto value = static_cast<jstring>(env->GetObjectArrayElement(arguments, index));
        if (env->ExceptionCheck()) return;
        strings[index] = utf8(env, value);
        env->DeleteLocalRef(value);
        if (env->ExceptionCheck()) return;
        args[index] = strings[index].c_str();
    }
    int result = mpv_command(player->mpv, args);
    if (result < 0) fail(env, std::string("mpv 命令失败: ") + mpv_error_string(result));
}

static std::string property(Player *player, const char *name) {
    char *value = mpv_get_property_string(player->mpv, name);
    std::string result = value ? value : "";
    mpv_free(value);
    return result;
}

extern "C" JNIEXPORT jobjectArray JNICALL
Java_com_loemby_mpv_MpvNative_snapshot(JNIEnv *env, jobject, jlong handle) {
    auto *player = reinterpret_cast<Player *>(handle);
    std::string values[9] = {"Loemby", "0", "0", "yes", "yes", "", "", "0", "0"};
    if (player && !player->shutdown) {
        values[0] = property(player, "media-title");
        values[1] = property(player, "time-pos");
        values[2] = property(player, "duration");
        values[3] = property(player, "pause");
        values[4] = "no";
        values[5] = property(player, "user-data/loemby-loading");
        values[6] = property(player, "user-data/loemby-ready");
        values[7] = property(player, "playlist-pos");
        values[8] = property(player, "playlist-count");
    }
    jobjectArray result = env->NewObjectArray(9, env->FindClass("java/lang/String"), nullptr);
    for (int index = 0; index < 9; ++index) {
        std::u16string text;
        try { text = std::wstring_convert<std::codecvt_utf8_utf16<char16_t>, char16_t>().from_bytes(values[index]); }
        catch (...) { text = u"\ufffd"; }
        jstring value = env->NewString(reinterpret_cast<const jchar *>(text.data()), static_cast<jsize>(text.size()));
        env->SetObjectArrayElement(result, index, value);
        env->DeleteLocalRef(value);
    }
    return result;
}

extern "C" JNIEXPORT void JNICALL
Java_com_loemby_mpv_MpvNative_resize(JNIEnv *env, jobject, jlong handle, jint width, jint height) {
    auto *player = reinterpret_cast<Player *>(handle);
    if (!player || player->shutdown) return;
    const std::string size = std::to_string(width) + "x" + std::to_string(height);
    int result = mpv_set_property_string(player->mpv, "android-surface-size", size.c_str());
    if (result < 0) __android_log_print(ANDROID_LOG_WARN, "LoembyMPV", "resize: %s", mpv_error_string(result));
}
