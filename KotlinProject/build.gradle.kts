import java.util.Properties

plugins {
    id("com.android.application") version "8.13.1"
    id("org.jetbrains.kotlin.android") version "2.0.21"
}

android {
    namespace = "com.arokodb"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.arokodb"
        minSdk = 24
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    ndkVersion = "28.2.13676358"

    sourceSets["main"].jniLibs.srcDir(layout.buildDirectory.dir("generated/jniLibs"))
}

val rustCrate = rootProject.projectDir.parentFile.resolve("rust/aroko-ffi")
val generatedJniLibs = layout.buildDirectory.dir("generated/jniLibs")
val localSdkPath = rootProject.file("local.properties").takeIf { it.isFile }?.let { propertiesFile ->
    Properties().apply {
        propertiesFile.inputStream().use { load(it) }
    }.getProperty("sdk.dir")
}
val sdkRoot = sequenceOf("ANDROID_HOME", "ANDROID_SDK_ROOT")
    .mapNotNull { System.getenv(it)?.takeIf(String::isNotBlank) }
    .firstOrNull()
    ?: localSdkPath
    ?: error("Set ANDROID_HOME or ANDROID_SDK_ROOT, or configure sdk.dir in local.properties.")
val ndkBin = File(sdkRoot, "ndk/28.2.13676358/toolchains/llvm/prebuilt/windows-x86_64/bin")

tasks.register("buildRustAndroid") {
    inputs.dir(rustCrate.resolve("src"))
    inputs.file(rustCrate.resolve("Cargo.toml"))
    inputs.file(rustCrate.resolve("Cargo.lock"))
    outputs.dir(generatedJniLibs)

    doLast {
        val targets = listOf(
            "arm64-v8a" to "aarch64-linux-android",
            "x86_64" to "x86_64-linux-android"
        )
        val outputRoot = generatedJniLibs.get().asFile
        outputRoot.deleteRecursively()

        targets.forEach { (abi, target) ->
            val linker = File(ndkBin, "${target}24-clang.cmd")
            check(linker.isFile) { "Android NDK linker was not found: $linker" }

            exec {
                workingDir = rustCrate
                environment("CARGO_TARGET_${target.replace('-', '_').uppercase()}_LINKER", linker.absolutePath)
                commandLine(
                    "cargo",
                    "build",
                    "--release",
                    "--target",
                    target,
                    "--manifest-path",
                    rustCrate.resolve("Cargo.toml").absolutePath
                )
            }

            val library = rustCrate.resolve("target/$target/release/libaroko_ffi.so")
            check(library.isFile) { "Rust Android library was not built: $library" }
            copy {
                from(library)
                into(File(outputRoot, abi))
            }
        }
    }
}

tasks.named("preBuild").configure {
    dependsOn("buildRustAndroid")
}