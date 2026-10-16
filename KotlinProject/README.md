# ArokoDB Android app

The Android app is a Kotlin UI backed by the same Rust exact-search core used
by the Windows demo. JNI passes vectors to Rust, which keeps the index in
memory and returns the nearest IDs and distances.

## Requirements

- Android Studio or an Android SDK with platform 35, build-tools 35, and NDK
  `28.2.13676358`.
- JDK 17.
- Rust with the Android targets installed:

  ```powershell
  rustup target add aarch64-linux-android x86_64-linux-android
  ```

The Android SDK must be discoverable through `ANDROID_HOME`,
`ANDROID_SDK_ROOT`, or `sdk.dir` in `local.properties`. The Gradle build
compiles the Rust JNI library for ARM64 phones and x86_64 emulators and
packages it in the APK automatically.

## Build and launch

From PowerShell in this directory:

```powershell
.\gradlew.bat assembleDebug
```

The APK is written to `build\outputs\apk\debug\ArokoDB-debug.apk`.

Start an emulator from Android Studio's Device Manager, or launch an existing
AVD from PowerShell:

```powershell
& "$env:ANDROID_HOME\emulator\emulator.exe" -avd ArokoDB_API35
```

With an emulator or Android device connected and USB debugging enabled:

```powershell
& "$env:ANDROID_HOME\platform-tools\adb.exe" wait-for-device
.\gradlew.bat installDebug
& "$env:ANDROID_HOME\platform-tools\adb.exe" shell am start -n com.arokodb/.MainActivity
```

To run the app later without rebuilding:

```powershell
& "$env:ANDROID_HOME\platform-tools\adb.exe" shell am start -n com.arokodb/.MainActivity
```

The same APK can be installed explicitly with:

```powershell
& "$env:ANDROID_HOME\platform-tools\adb.exe" install -r .\build\outputs\apk\debug\ArokoDB-debug.apk
```

## Using the app

1. Set a dimension and select **Create / reset index**. This clears the
   current in-memory index.
2. Give a vector a unique non-negative ID and enter exactly that many
   comma-separated finite numbers, then select **Add vector**.
3. Enter a query with the same dimension, choose a distance metric and Top K,
   then select **Search**. Lower distances indicate closer vectors.

The index is exact and in-memory. Closing the app discards its vectors; this
Android preview does not yet import/export CSV or persist data.

## Rebuild and diagnostics

Gradle rebuilds the Rust native library when its crate inputs change. To force
a clean rebuild and recreate the APK:

```powershell
.\gradlew.bat clean
.\gradlew.bat assembleDebug
```

To inspect Android runtime errors:

```powershell
& "$env:ANDROID_HOME\platform-tools\adb.exe" logcat -s AndroidRuntime
```
