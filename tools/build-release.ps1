param(
    [ValidateSet('ANSI', 'ISO', 'JIS', 'KR')]
    [string]$Layout = 'ANSI'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$layoutLower = $Layout.ToLowerInvariant()
$featureArguments = @('--no-default-features', '--features', ('layout-{0}' -f $layoutLower))
$firmwareDir = if ($Layout -eq 'ANSI') {
    Join-Path $repo 'firmware'
}
else {
    Join-Path $repo 'firmware\experimental'
}
$targetDir = Join-Path $repo 'target\thumbv7em-none-eabihf\release'
$nrfutil = Get-Command adafruit-nrfutil -ErrorAction Stop

Push-Location $repo
try {
    cargo fmt --package nocfree-and-rust -- --check
    $fmtExitCode = $LASTEXITCODE
    if ($fmtExitCode -ne 0) {
        throw ('Rust format check failed (exit {0})' -f $fmtExitCode)
    }

    cargo test --target x86_64-pc-windows-msvc --package nocfree-and-rust @featureArguments
    $testExitCode = $LASTEXITCODE
    if ($testExitCode -ne 0) {
        throw ('Host tests failed (exit {0})' -f $testExitCode)
    }

    cargo clippy --target x86_64-pc-windows-msvc --lib @featureArguments -- -D warnings
    $hostClippyExitCode = $LASTEXITCODE
    if ($hostClippyExitCode -ne 0) {
        throw ('Host clippy failed (exit {0})' -f $hostClippyExitCode)
    }

    foreach ($name in @('central', 'right')) {
        cargo clippy --release --bin $name @featureArguments -- -D warnings
        $clippyExitCode = $LASTEXITCODE
        if ($clippyExitCode -ne 0) {
            throw ('Clippy failed for {0} (exit {1})' -f $name, $clippyExitCode)
        }

        cargo build --release --bin $name @featureArguments
        $buildExitCode = $LASTEXITCODE
        if ($buildExitCode -ne 0) {
            throw ('Release build failed for {0} (exit {1})' -f $name, $buildExitCode)
        }
    }

    $sysroot = rustc --print sysroot
    $rustcExitCode = $LASTEXITCODE
    if ($rustcExitCode -ne 0) {
        throw ('Rust sysroot query failed (exit {0})' -f $rustcExitCode)
    }
    $versionInfo = rustc -vV
    $versionExitCode = $LASTEXITCODE
    if ($versionExitCode -ne 0) {
        throw ('Rust host query failed (exit {0})' -f $versionExitCode)
    }
    $hostLine = $versionInfo | Where-Object { $_ -like 'host: *' }
    if ($null -eq $hostLine) {
        throw 'Rust host triple was not reported'
    }
    $hostTriple = $hostLine.Substring(6)
    $llvmObjcopy = Join-Path $sysroot ('lib\rustlib\{0}\bin\llvm-objcopy.exe' -f $hostTriple)
    if (-not (Test-Path -LiteralPath $llvmObjcopy)) {
        throw 'llvm-tools-preview is missing; run: rustup component add llvm-tools-preview'
    }

    New-Item -ItemType Directory -Path $firmwareDir -Force | Out-Null
    $python = Get-Command python -ErrorAction Stop

    $artifacts = @(
        @{ Binary = 'central'; Half = 'Left' },
        @{ Binary = 'right'; Half = 'Right' }
    )
    foreach ($artifact in $artifacts) {
        $binaryName = $artifact.Binary
        $halfName = $artifact.Half
        $elf = Join-Path $targetDir $binaryName
        if ($Layout -eq 'ANSI') {
            $bin = Join-Path $firmwareDir ('NocFree_Rust_{0}.bin' -f $halfName)
            $uf2 = Join-Path $firmwareDir ('NocFree_And_Rust_ZMK_Based_ANSI_{0}.uf2' -f $halfName)
            $dfu = Join-Path $firmwareDir ('NocFree_Rust_{0}_DFU.zip' -f $halfName)
        }
        else {
            $stem = 'NocFree_And_Rust_ZMK_Based_{0}_Experimental_{1}' -f $Layout, $halfName
            $bin = Join-Path $firmwareDir ('{0}.bin' -f $stem)
            $uf2 = Join-Path $firmwareDir ('{0}.uf2' -f $stem)
            $dfu = Join-Path $firmwareDir ('{0}_DFU.zip' -f $stem)
        }
        if (-not (Test-Path -LiteralPath $elf)) {
            throw ('Missing release ELF {0}' -f $elf)
        }

        & $llvmObjcopy -O binary $elf $bin
        $objcopyExitCode = $LASTEXITCODE
        if ($objcopyExitCode -ne 0) {
            throw ('Binary extraction failed for {0} (exit {1})' -f $halfName, $objcopyExitCode)
        }

        & $python.Source -B (Join-Path $repo 'tools\nocfree_uf2.py') $bin $uf2
        $uf2ExitCode = $LASTEXITCODE
        if ($uf2ExitCode -ne 0) {
            throw ('UF2 packing failed for {0} (exit {1})' -f $halfName, $uf2ExitCode)
        }

        & $nrfutil.Source dfu genpkg --application $bin --dev-type 82 --dfu-ver 0.5 --sd-req 0xFFFE $dfu
        $dfuExitCode = $LASTEXITCODE
        if ($dfuExitCode -ne 0) {
            throw ('Serial DFU packing failed for {0} (exit {1})' -f $halfName, $dfuExitCode)
        }
    }

    & $python.Source -B -m unittest discover -s tools -p 'test_*.py'
    $pythonTestExitCode = $LASTEXITCODE
    if ($pythonTestExitCode -ne 0) {
        throw ('Artifact tests failed (exit {0})' -f $pythonTestExitCode)
    }

    Get-ChildItem -LiteralPath $firmwareDir -File |
        Select-Object Name, Length, FullName

    Write-Output ('NocFree {0} release verification passed' -f $Layout)
}
finally {
    Pop-Location
}
