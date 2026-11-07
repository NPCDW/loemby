buildscript {
    repositories {
        google()
        mavenCentral()
    }
    dependencies {
        // API 36 requires AGP 8.10+; keep the wrapper at its supported Gradle version.
        classpath("com.android.tools.build:gradle:8.10.1")
        classpath("org.jetbrains.kotlin:kotlin-gradle-plugin:1.9.25")
    }
}

allprojects {
    repositories {
        google()
        mavenCentral()
    }
}

tasks.register("clean").configure {
    delete("build")
}

