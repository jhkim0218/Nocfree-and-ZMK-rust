$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$firmwareDir = Join-Path $repo 'firmware'
$targetDir = Join-Path $repo 'target\thumbv7em-none-eabihf\release'

Push-Location $repo
try {
    cargo fmt --package nocfree-and-rust -- --check
    $fmtExitCode = $LASTEXITCODE
    if ($fmtExitCode -ne 0) {
        throw ('Rust format check failed (exit {0})' -f $fmtExitCode)
    }

    cargo test --target x86_64-pc-windows-msvc --package nocfree-and-rust
    $testExitCode = $LASTEXITCODE
    if ($testExitCode -ne 0) {
        throw ('Host tests failed (exit {0})' -f $testExitCode)
    }

    $python = Get-Command python -ErrorAction Stop
    & $python.Source -B -m unittest discover -s tools -p 'test_*.py'
    $pythonTestExitCode = $LASTEXITCODE
    if ($pythonTestExitCode -ne 0) {
        throw ('UF2 tests failed (exit {0})' -f $pythonTestExitCode)
    }

    cargo clippy --target x86_64-pc-windows-msvc --lib -- -D warnings
    $hostClippyExitCode = $LASTEXITCODE
    if ($hostClippyExitCode -ne 0) {
        throw ('Host clippy failed (exit {0})' -f $hostClippyExitCode)
    }

    foreach ($name in @('central', 'right')) {
        cargo clippy --release --bin $name -- -D warnings
        $clippyExitCode = $LASTEXITCODE
        if ($clippyExitCode -ne 0) {
            throw ('Clippy failed for {0} (exit {1})' -f $name, $clippyExitCode)
        }

        cargo build --release --bin $name
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

    $artifacts = @(
        @{ Binary = 'central'; Half = 'Left' },
        @{ Binary = 'right'; Half = 'Right' }
    )
    foreach ($artifact in $artifacts) {
        $binaryName = $artifact.Binary
        $halfName = $artifact.Half
        $elf = Join-Path $targetDir $binaryName
        $bin = Join-Path $firmwareDir ('NocFree_Rust_{0}.bin' -f $halfName)
        $uf2 = Join-Path $firmwareDir ('NocFree_Rust_{0}.uf2' -f $halfName)
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
    }

    Get-ChildItem -LiteralPath $firmwareDir -File |
        Where-Object { $_.Name -like 'NocFree_Rust_*' } |
        Select-Object Name, Length, FullName
}
finally {
    Pop-Location
}
