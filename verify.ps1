# ============================================================================
# ArokoDB Android App Build + Run Script (PowerShell)
# Builds the Kotlin/Android app and lets you choose which device/emulator to run on
# ============================================================================

Write-Host "`n=== ArokoDB Android App Builder + Runner === `n" -ForegroundColor Green
Write-Host ""

# ---- Configuration ----
$ProjectDir = Get-Location
$AndroidSdk = $env:ANDROID_SDK -or "$env:USERPROFILE\AppData\Local\Android\Sdk"
$Gradle = Join-Path $AndroidSdk "gradle/bin/gradle"
$Adb = Join-Path $AndroidSdk "platform-tools/adb"

Write-Host "Project dir: $ProjectDir" -ForegroundColor Cyan
Write-Host "Android SDK: $AndroidSdk" -ForegroundColor Cyan
Write-Host ""

# ---- Step 1: Build the app ----
Write-Host "[1/4] Building application..." -ForegroundColor Yellow

if (-not (Test-Path $Gradle)) {
    Write-Host "ERROR: Gradle not found at $Gradle" -ForegroundColor Red
    Write-Host "Please set ANDROID_SDK or install Android SDK."
    return
}

# Gradle sync + assembleDebug
Write-Host "Running Gradle sync..."
& $Gradle sync > $null 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Gradle sync failed!" -ForegroundColor Red
    return
}

Write-Host "Running assembleDebug..."
& $Gradle assembleDebug > $null 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    return
}

$ApkPath = Join-Path $ProjectDir "app/build/outputs/apk/debug/app-debug.apk"
if (-not (Test-Path $ApkPath)) {
    Write-Host "ERROR: APK not found at $ApkPath" -ForegroundColor Red
    return
}

Write-Host "âœ“ Build successful" -ForegroundColor Green
Write-Host ""

# ---- Step 2: List connected devices ----
Write-Host "[2/4] Detecting connected devices..." -ForegroundColor Yellow

$Devices = & $Adb devices -l 2>/dev/null

if (-not $Devices) {
    Write-Host "No devices detected via ADB!" -ForegroundColor Red
    Write-Host "Make sure USB debugging is enabled on device, or emulator is running." -ForegroundColor Cyan
    return
}

# Parse devices - skip first header line
$DeviceIds = @()
$DeviceNames = @()
$Count = 0

# Split by newlines and process each line
$Lines = $Devices -split "`n"
# Skip first line (header)
for ($i = 1; $i -le $Lines.Count; $i++) {
    $Line = $Lines[$i]
    if (-not $Line) { continue }

    # Format: "XXXXXXXXXXXX    device product:model transport_id:net_socket:"
    # Or just: "emulator-5554   device"
    $Parts = $Line -split `\s+`
    if ($Parts.Count -lt 1) { continue }

    $DeviceId = $Parts[0]
    if (-not $DeviceId) { continue }

    # Extract model from -l flag
    $DeviceLabel = "Physical Device"
    if ($Line -match "model (.+)") {
        $DeviceLabel = $Matches[1]
    }

    $DeviceIds += $DeviceId
    $DeviceNames += $DeviceLabel
    $Count++
}

Write-Host "Found $Count device(s):" -ForegroundColor Cyan
for ($i = 0; $i -lt $DeviceIds.Count; $i++) {
    Write-Host "$((i+1))) ${DeviceNames[$i]} (${DeviceIds[$i]})"
}

# ---- Step 3: Let user choose device ----
Write-Host "" -ForegroundColor Yellow
Write-Host "[3/4] Select device to run on:" -ForegroundColor Yellow

# Build choice list
$Choices = @()
for ($i = 0; $i -lt $DeviceIds.Count; $i++) {
    $Choices += "$((i+1))) ${DeviceNames[$i]} (${DeviceIds[$i]})"
}
$Choices += "r) Random device"
$Choices += "q) Quit"

# Use Read-Host with choice prompt
$ChoosenId = $null
do {
    Write-Host "Your choice (1-$($DeviceIds.Count), r=random, q=quit):"
    $Choice = Read-Host

    switch (
        $Choice
    ) {
        "q" { Write-Host "Exiting." -ForegroundColor Cyan; return }
        "r" {
            $RandIdx = Get-Random 0 ($DeviceIds.Count)
            $ChoosenId = $DeviceIds[$RandIdx]
            Write-Host "Selected random device: $ChoosenId" -ForegroundColor Green
            break
        }
        default {
            try {
                $Num = [int]$Choice
                if ($Num -ge 1 -and $Num -le $DeviceIds.Count) {
                    $Idx = $Num - 1
                    $ChoosenId = $DeviceIds[$Idx]
                    Write-Host "Selected: $Choice" -ForegroundColor Green
                } else {
                    Write-Host "Number out of range. Try again." -ForegroundColor Yellow
                }
            } else {
                Write-Host "Invalid selection. Try again." -ForegroundColor Yellow
            }
        }
    }

    if ($ChoosenId) { break }
    }
}

if (-not $ChoosenId) {
    Write-Host "No device selected. Exiting." -ForegroundColor Red
    return
}

Write-Host "" -ForegroundColor Yellow
Write-Host "[4/4] Installing and launching on $ChoosenId..." -ForegroundColor Yellow

# ---- Step 4: Install and launch ----
# Install the APK
Write-Host "Installing APK..."

$InstallOutput = & $Adb -s $ChoosenId install -r $ApkPath 2>&1
Write-Host "Install output: $InstallOutput"

# Launch the MainActivity
$Activity = "com.arokodb.vectordb/.MainActivity"
Write-Host "Launching $Activity..."

$LaunchOutput = & $Adb -s $ChoosenId shell am start -n $Activity 2>&1
Write-Host "Launch output: $LaunchOutput"

Write-Host "" -ForegroundColor Green
Write-Host "âœ“ Done! App running on device: $ChoosenId" -ForegroundColor Green
Write-Host ""
Write-Host "The ArokoDB vector database app is now running."
Write-Host "Use 'adb -s $ChoosenId logcat' to see console output."
Write-Host "Press Ctrl+C to exit this script." -ForegroundColor Cyan
