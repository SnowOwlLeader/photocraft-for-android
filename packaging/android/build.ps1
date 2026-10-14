param(
    [ValidateSet('arm64-v8a','x86_64')][string]$Abi = 'arm64-v8a',
    [string]$ToolsRoot = 'E:\photocraft-build',
    [ValidateRange(1,32)][int]$Jobs = 2,
    [switch]$SkipRust
)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\env.ps1" -ToolsRoot $ToolsRoot
$root = (Resolve-Path "$PSScriptRoot\..\..").Path
$target = if ($Abi -eq 'arm64-v8a') { 'aarch64-linux-android' } else { 'x86_64-linux-android' }
# Separate invocations must never rewrite each other's manifest/dex/signing inputs.
$build = Join-Path $ToolsRoot ("apk-{0}-{1}" -f $Abi, [Guid]::NewGuid().ToString('N'))
$bt = "$env:ANDROID_HOME\build-tools\35.0.0"
$platform = "$env:ANDROID_HOME\platforms\android-35\android.jar"
function Check-Exit { if ($LASTEXITCODE -ne 0) { throw "Build tool failed with exit code $LASTEXITCODE" } }
Push-Location $root
try {
    if (-not $SkipRust) {
        $env:CARGO_PROFILE_RELEASE_LTO = 'false'
        $env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '16'
        cargo build --locked -p photocraft-android --target $target --release -j $Jobs
        Check-Exit
    }
    New-Item -ItemType Directory -Force "$build\classes", "$build\dex", "$build\res\drawable", "$root\dist\android", "$ToolsRoot\signing" | Out-Null
    Copy-Item -LiteralPath "$root\assets\app-icon\hicolor\256x256\apps\ai.storyteller.photocraft.png" -Destination "$build\res\drawable\icon.png" -Force
    $javaSources = Get-ChildItem -LiteralPath "$PSScriptRoot\java\ai\storyteller\photocraft" -Filter '*.java' | Select-Object -ExpandProperty FullName
    & "$env:JAVA_HOME\bin\javac.exe" -encoding UTF-8 --release 8 -classpath $platform -d "$build\classes" $javaSources
    Check-Exit
    & "$env:JAVA_HOME\bin\jar.exe" cf "$build\classes.jar" -C "$build\classes" .
    Check-Exit
    & "$bt\d8.bat" --release --min-api 28 --lib $platform --output "$build\dex" "$build\classes.jar"
    Check-Exit
    & "$bt\aapt2.exe" compile --dir "$build\res" -o "$build\resources.zip"
    Check-Exit
    & "$bt\aapt2.exe" link -I $platform --manifest "$PSScriptRoot\AndroidManifest.xml" -o "$build\unsigned.apk" "$build\resources.zip"
    Check-Exit
    $env:PHOTOCRAFT_APK_BUILD = $build
    $env:PHOTOCRAFT_APK_ABI = $Abi
    $env:PHOTOCRAFT_APK_LIB = "$env:CARGO_TARGET_DIR\$target\release\libphotocraft_android.so"
    $env:PHOTOCRAFT_SOURCE_ROOT = $root
    @'
import os,zipfile
from pathlib import Path
b=Path(os.environ['PHOTOCRAFT_APK_BUILD'])
root=Path(os.environ['PHOTOCRAFT_SOURCE_ROOT'])
with zipfile.ZipFile(b/'unsigned.apk','a',compression=zipfile.ZIP_DEFLATED) as z:
    z.write(b/'dex/classes.dex','classes.dex')
    z.write(os.environ['PHOTOCRAFT_APK_LIB'],'lib/'+os.environ['PHOTOCRAFT_APK_ABI']+'/libphotocraft_android.so')
    for p in ['LICENSE-MIT','LICENSE-APACHE','ATTRIBUTION.md','assets/app-icon/LICENSE.txt','vendor/winit/LICENSE']:
        if (root/p).exists(): z.write(root/p,'assets/licenses/'+p.replace('/','_'))
'@ | python -
    Check-Exit
    & "$bt\zipalign.exe" -f -P 16 4 "$build\unsigned.apk" "$build\aligned.apk"
    Check-Exit
    $key = "$ToolsRoot\signing\photocraft-local.p12"
    $passwordFile = "$ToolsRoot\signing\password.txt"
    if (-not (Test-Path -LiteralPath $key)) {
        $bytes = New-Object byte[] 32
        [System.Security.Cryptography.RandomNumberGenerator]::Fill($bytes)
        [IO.File]::WriteAllText($passwordFile, [Convert]::ToBase64String($bytes))
        & "$env:JAVA_HOME\bin\keytool.exe" -genkeypair -keystore $key -storetype PKCS12 -storepass:file $passwordFile -alias photocraft-local -keyalg RSA -keysize 3072 -validity 10000 -dname 'CN=PhotoCraft Local Android Build'
        Check-Exit
    }
    $apk = "$root\dist\android\PhotoCraft-0.8.2-$Abi.apk"
    & "$bt\apksigner.bat" sign --ks $key --ks-key-alias photocraft-local --ks-pass "file:$passwordFile" --out $apk "$build\aligned.apk"
    Check-Exit
    & "$bt\apksigner.bat" verify --verbose $apk
    Check-Exit
    & "$bt\zipalign.exe" -c -P 16 4 $apk
    Check-Exit
    Get-FileHash -Algorithm SHA256 -LiteralPath $apk | Format-List
    Write-Output "APK: $apk"
} finally { Pop-Location }
