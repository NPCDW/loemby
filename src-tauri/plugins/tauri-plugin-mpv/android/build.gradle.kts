plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
}

val nativeDir = file(System.getenv("LOEMBY_MPV_NATIVE_DIR") ?: "../native")
val ndkRelease = "27.2.12479018"

android {
    namespace = "com.loemby.mpv"
    compileSdk = 36
    ndkVersion = ndkRelease
    defaultConfig {
        minSdk = 24
        consumerProguardFiles("consumer-rules.pro")
        ndk { abiFilters.add("arm64-v8a") }
        externalNativeBuild {
            cmake {
                arguments += listOf("-DMPV_NATIVE_DIR=${nativeDir.absolutePath}", "-DANDROID_STL=c++_shared", "-DANDROID_SUPPORT_FLEXIBLE_PAGE_SIZES=ON")
            }
        }
    }
    externalNativeBuild { cmake { path = file("src/main/cpp/CMakeLists.txt") } }
    sourceSets.getByName("main").jniLibs.srcDir(layout.buildDirectory.dir("generated/mpv-jni"))
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    kotlinOptions { jvmTarget = "1.8" }
}

// Imported CMake libraries are packaged by externalNativeBuild. Other dependency
// libraries come from jniLibs; the app supplies the matching libc++ runtime once.
val checkMpvNative by tasks.registering(Exec::class) {
    val script = file("../../../../scripts/android-mpv.py")
    commandLine("python3", "-I", script.absolutePath, "check", "--bundle", nativeDir.absolutePath, "--abi", "arm64-v8a")
}
val syncMpvDependencies by tasks.registering(Sync::class) {
    dependsOn(checkMpvNative)
    from(nativeDir.resolve("jniLibs")) {
        exclude("**/libmpv.so", "**/libavcodec.so", "**/libc++_shared.so")
    }
    into(layout.buildDirectory.dir("generated/mpv-jni"))
}
tasks.named("preBuild") { dependsOn(syncMpvDependencies) }

dependencies {
    implementation(project(":tauri-android"))
}
