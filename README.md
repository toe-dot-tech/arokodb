# ArokoDB

ArokoDB is an in-progress embedded vector-search project. This repository
currently contains a runnable **Windows desktop demo** backed by a Rust exact
vector-search baseline and an **Android app connected to the same Rust engine
through JNI**.

The portfolio target (100,000 vectors, HNSW, mmap, low memory/latency, and
multiple platforms) is a design goal, not a result demonstrated by the current
implementation. The Windows demo supports configurable dimensions, unique
vector IDs, CSV import/export, top-K search, and squared Euclidean, cosine, and
Manhattan distance. Vectors live only in memory until exported, and search
scans every vector, so results are exact but the index is not HNSW.

## Requirements

- Windows 10/11, 64-bit, for the desktop UI.
- Rust installed with the `x86_64-pc-windows-msvc` target. Install the
  **MSVC C++ Build Tools** if Rust reports that the linker is missing.
- Android Studio, Android SDK platform/build-tools 35, NDK
  `28.2.13676358`, JDK 17, and Rust Android targets for Android development;
  see [Android app](#android-app).

## Windows desktop app

The Windows app opens into **Vector Studio**, a clear workspace for entering
vectors, searching for similar items, and importing or exporting CSV. Its
**Industry ideas** section explains possible uses in banking and finance,
cryptocurrency, healthcare, news and digital media, AI, social and community,
fitness and wellness, and education. These are product ideas told in everyday
language, not claims that the app is already connected to those industries.

Run these commands in PowerShell from the repository root:

```powershell
cargo test --manifest-path rust\aroko-core\Cargo.toml
cargo test --manifest-path rust\aroko-windows\Cargo.toml
cargo run --release --manifest-path rust\aroko-windows\Cargo.toml
```

The final command builds the app if needed and launches it. To rebuild the
standalone release executable without launching the window:

```powershell
cargo build --release --manifest-path rust\aroko-windows\Cargo.toml
```

The updated executable is written to:

```text
rust\aroko-windows\target\release\aroko-windows.exe
```

Launch the already-built executable from PowerShell:

```powershell
.\rust\aroko-windows\target\release\aroko-windows.exe
```

Cargo automatically rebuilds when source files or dependencies have changed.
To force a clean rebuild if the executable seems stale, run:

```powershell
cargo clean --manifest-path rust\aroko-windows\Cargo.toml
cargo build --release --manifest-path rust\aroko-windows\Cargo.toml
```

`cargo clean` removes generated build output for this crate; it does not remove
source files. Do not copy the executable alone to another computer unless its
Rust/MSVC runtime requirements are also satisfied.

### Using the demo

1. Set **Vector dimension** (default `3`) and select **Apply dimension** before
   adding vectors. To change the dimension after adding data, select **Clear
   vectors** first; clearing permanently removes the current in-memory data.
2. Enter a unique non-negative **Vector ID** and exactly that many comma-
   separated values, for example `0.1, 0.2, 0.3`; then select **Add vector**.
3. Enter a query with the same number of values, choose **Top K** and a
   **Distance metric**, then select **Search**. Results are sorted closest
   first. Cosine distance is `1 - cosine similarity`; lower scores are closer
   for all three metrics.
4. To save or load data, enter a CSV path (default `vectors.csv`) and select
   **Export CSV** or **Import CSV**. Relative paths use the process's current
   working directory. Imports are rejected if the header/dimension is wrong,
   IDs repeat, or an ID already exists in the current index.

CSV uses one vector per row, with an `id` column followed by dimension columns.
For a three-dimensional index:

```csv
id,v1,v2,v3
10,0.1,0.2,0.3
11,1,0,0
```

CSV fields are plain comma-separated numbers (quoted fields are not supported).
To avoid losing changes, export before closing the app.

From **Industry ideas**, choose **Try this example** to load three small
illustrative vectors and run a sample search in Vector Studio. These made-up
numbers demonstrate the controls; they are not real text embeddings or
measured industry results.

## Android app

The Android project is a Kotlin UI connected to the Rust exact-search core by
a JNI bridge. It supports configurable vector dimensions, custom IDs,
nearest-neighbor search, and squared Euclidean, cosine, and Manhattan distance.
The index is exact and in-memory; the Android preview does not yet persist
vectors or support CSV import/export.

From PowerShell at the repository root, build the debug APK:

```powershell
cd KotlinProject
.\gradlew.bat assembleDebug
```

The APK is written to
`KotlinProject\build\outputs\apk\debug\ArokoDB-debug.apk`. Start an emulator
from Android Studio, then install and launch the app:

```powershell
& "$env:ANDROID_HOME\platform-tools\adb.exe" wait-for-device
.\gradlew.bat installDebug
& "$env:ANDROID_HOME\platform-tools\adb.exe" shell am start -n com.arokodb/.MainActivity
```

See [KotlinProject/README.md](KotlinProject/README.md) for Android setup,
emulator, rebuild, and troubleshooting instructions.

## Repository layout

```text
rust\
  aroko-core\       Rust exact-search core shared by clients
  aroko-windows\    Native Windows demo application
  aroko-ffi\        JNI bridge used by the Android app
  aroko-storage\    Incomplete storage/mmap prototype
KotlinProject\      Kotlin Android app and Gradle build
```

Keep the Windows UI and Android UI as separate platform clients, with the
vector engine in Rust. The Windows Rust executable calls the core directly,
while Android calls it through JNI. Do not describe the storage prototype as
mmap-backed or the index as HNSW until those implementations have been
completed and tested.

## Current project limitations

- Search is exact and linear in the number of stored vectors.
- The Windows demo keeps data in memory while open; CSV export/import provides
  manual persistence. It does not use FFI.
- Android uses the Rust exact-search core through JNI, but vectors are
  in-memory only and are discarded when the app process ends.
- The Rust storage and FFI crates are prototypes and are not part of the
  working Windows application.
- No 100,000-vector latency, memory, or cross-platform benchmark is included.
