import java.io.FileInputStream
import java.util.Properties
import groovy.json.JsonSlurper

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("rust")
}

val rustlsVerifierPackage = run {
    val metadataText = providers.exec {
        workingDir(file("../../.."))
        commandLine(
            "cargo", "metadata", "--format-version", "1", "--locked", "--offline",
            "--filter-platform", "aarch64-linux-android",
            "--manifest-path", file("../../../Cargo.toml").absolutePath
        )
    }.standardOutput.asText.get()
    val metadata = JsonSlurper().parseText(metadataText) as Map<*, *>
    val packages = metadata["packages"] as List<*>
    packages.map { it as Map<*, *> }.single { it["name"] == "rustls-platform-verifier-android" }
}
val rustlsVerifierVersion = rustlsVerifierPackage["version"] as String
val rustlsVerifierMavenDir = file(rustlsVerifierPackage["manifest_path"] as String).parentFile.resolve("maven")

repositories {
    maven {
        name = "rustlsPlatformVerifier"
        url = uri(rustlsVerifierMavenDir)
        metadataSources { mavenPom(); artifact() }
        content { includeGroup("rustls") }
    }
}

val tauriProperties = Properties().apply {
    val propFile = file("tauri.properties")
    if (propFile.exists()) {
        propFile.inputStream().use { load(it) }
    }
}

val tauriConfig = JsonSlurper().parse(file("../../../tauri.conf.json")) as Map<*, *>
val appVersion = tauriConfig["version"] as String
val versionParts = appVersion.substringBefore('-').split('.').map(String::toInt)
val appVersionCode = versionParts[0] * 1_000_000 + versionParts[1] * 1_000 + versionParts[2]

android {
    compileSdk = 36
    ndkVersion = "27.2.12479018"
    buildToolsVersion = "36.0.0"
    namespace = "com.loemby.app"
    defaultConfig {
        manifestPlaceholders["usesCleartextTraffic"] = "false"
        applicationId = "com.loemby.app"
        minSdk = 24
        targetSdk = 36
        ndk {
            abiFilters.clear()
            abiFilters += "arm64-v8a"
        }
        versionCode = tauriProperties.getProperty("tauri.android.versionCode", appVersionCode.toString()).toInt()
        versionName = tauriProperties.getProperty("tauri.android.versionName", appVersion)
    }
    signingConfigs {
        val keystorePropertiesFile = rootProject.file("keystore.properties")
        if (keystorePropertiesFile.exists()) {
            create("release") {
                val keystoreProperties = Properties().apply {
                    FileInputStream(keystorePropertiesFile).use { load(it) }
                }
                keyAlias = keystoreProperties.getProperty("keyAlias")
                    ?: error("keystore.properties is missing keyAlias")
                keyPassword = keystoreProperties.getProperty("password")
                    ?: error("keystore.properties is missing password")
                storeFile = file(keystoreProperties.getProperty("storeFile")
                    ?: error("keystore.properties is missing storeFile"))
                storePassword = keystoreProperties.getProperty("password")
            }
        }
    }
    buildTypes {
        getByName("debug") {
            manifestPlaceholders["usesCleartextTraffic"] = "true"
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {
                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            isMinifyEnabled = true
            proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
            // Manual verification CI produces an unsigned release when no private keystore is supplied.
            signingConfig = signingConfigs.findByName("release")
        }
    }
    kotlinOptions {
        jvmTarget = "1.8"
    }
    buildFeatures {
        buildConfig = true
    }
}

val syncAndroidIcons by tasks.registering(Copy::class) {
    from(file("../../../icons/android")) {
        include("mipmap-*/ic_launcher*.png")
    }
    into(file("src/main/res"))
}

tasks.named("preBuild") {
    dependsOn(syncAndroidIcons)
}

rust {
    rootDirRel = "../../../"
}

dependencies {
    implementation("rustls:rustls-platform-verifier:$rustlsVerifierVersion")
    implementation("androidx.webkit:webkit:1.6.1")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation("com.google.android.material:material:1.8.0")
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.1.4")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.5.0")
}

apply(from = "tauri.build.gradle.kts")