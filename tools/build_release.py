from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
ARM_TARGET = "thumbv7em-none-eabihf"
LAYOUTS = ("ANSI", "ISO", "JIS", "KR")
BACKLIGHT_CURVES = ("linear", "perceptual")


def run(*command: str) -> None:
    print("+", subprocess.list2cmdline(command), flush=True)
    subprocess.run(command, cwd=ROOT, check=True)


def output(command: str, *arguments: str) -> str:
    return subprocess.check_output(
        (command, *arguments), cwd=ROOT, text=True
    ).strip()


def rust_host() -> str:
    for line in output("rustc", "-vV").splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ")
    raise RuntimeError("rustc did not report its host triple")


def llvm_tool(name: str) -> Path:
    sysroot = Path(output("rustc", "--print", "sysroot"))
    path = sysroot / "lib" / "rustlib" / rust_host() / "bin" / name
    if sys.platform == "win32":
        path = path.with_suffix(".exe")
    if not path.is_file():
        raise RuntimeError(
            "llvm-tools-preview is missing; run: rustup component add llvm-tools-preview"
        )
    return path


def llvm_objcopy() -> Path:
    return llvm_tool("llvm-objcopy")


def artifact_paths(
    layout: str, half: str, backlight_curve: str = "perceptual"
) -> tuple[Path, Path, Path]:
    directory = ROOT / "firmware"
    if layout == "ANSI" and backlight_curve == "perceptual":
        return (
            directory / f"NocFree_Rust_{half}.bin",
            directory / f"NocFree_And_Rust_ZMK_Based_ANSI_{half}.uf2",
            directory / f"NocFree_Rust_{half}_DFU.zip",
        )
    directory /= "experimental"
    if layout == "ANSI":
        stem = f"NocFree_And_Rust_ZMK_Based_ANSI_Linear_Backlight_Experimental_{half}"
    else:
        stem = f"NocFree_And_Rust_ZMK_Based_{layout}_Experimental_{half}"
    return (
        directory / f"{stem}.bin",
        directory / f"{stem}.uf2",
        directory / f"{stem}_DFU.zip",
    )


def dongle_artifact_paths() -> tuple[Path, Path, Path]:
    stem = "NocFree_And_Rust_ZMK_Based_ANSI_Dongle_D1"
    directory = ROOT / "firmware"
    return (
        directory / f"{stem}.bin",
        directory / f"{stem}.uf2",
        directory / f"{stem}_DFU.zip",
    )


def build_dongle(host: str, objcopy: Path, nrfutil: str) -> None:
    features = (
        "--no-default-features",
        "--features",
        "layout-ansi,backlight-perceptual,standalone-critical-section",
    )
    run("cargo", "test", "--target", host, "--package", "nocfree-and-rust", *features)
    run("cargo", "clippy", "--target", host, "--lib", *features, "--", "-D", "warnings")
    run(
        "cargo",
        "clippy",
        "--release",
        "--target",
        ARM_TARGET,
        "--bin",
        "dongle",
        *features,
        "--",
        "-D",
        "warnings",
    )
    run(
        "cargo",
        "build",
        "--release",
        "--target",
        ARM_TARGET,
        "--bin",
        "dongle",
        *features,
    )

    elf = ROOT / "target" / ARM_TARGET / "release" / "dongle"
    symbols = output(str(llvm_tool("llvm-nm")), "--numeric-sort", str(elf))
    if any(name in symbols for name in ("nrf_softdevice", "sd_ble", "sd_radio")):
        raise RuntimeError("dongle ELF contains SoftDevice radio symbols")
    symbol_addresses = {
        fields[2]: fields[0]
        for line in symbols.splitlines()
        if len(fields := line.split(maxsplit=2)) == 3
    }
    if symbol_addresses.get("RADIO") != symbol_addresses.get("DefaultHandler"):
        raise RuntimeError("dongle RADIO vector is not the unused default handler")

    binary_path, uf2_path, dfu_path = dongle_artifact_paths()
    run(str(objcopy), "-O", "binary", str(elf), str(binary_path))
    run(
        sys.executable,
        "-B",
        str(ROOT / "tools" / "nocfree_uf2.py"),
        str(binary_path),
        str(uf2_path),
    )
    dfu_path.unlink(missing_ok=True)
    run(
        nrfutil,
        "dfu",
        "genpkg",
        "--application",
        str(binary_path),
        "--dev-type",
        "82",
        "--dfu-ver",
        "0.5",
        "--sd-req",
        "0x0123",
        str(dfu_path),
    )
    print("NocFree ANSI dongle D1 release verification passed", flush=True)


def build_layout(
    layout: str,
    host: str,
    objcopy: Path,
    nrfutil: str,
    backlight_curve: str = "perceptual",
) -> None:
    selected_features = [f"layout-{layout.lower()}", "split-softdevice"]
    if backlight_curve == "perceptual":
        selected_features.append("backlight-perceptual")
    features = (
        "--no-default-features",
        "--features",
        ",".join(selected_features),
    )
    run(
        "cargo",
        "test",
        "--target",
        host,
        "--package",
        "nocfree-and-rust",
        *features,
    )
    run("cargo", "clippy", "--target", host, "--lib", *features, "--", "-D", "warnings")

    target_directory = ROOT / "target" / ARM_TARGET / "release"
    for binary, half in (("central", "Left"), ("right", "Right")):
        run(
            "cargo",
            "clippy",
            "--release",
            "--target",
            ARM_TARGET,
            "--bin",
            binary,
            *features,
            "--",
            "-D",
            "warnings",
        )
        run(
            "cargo",
            "build",
            "--release",
            "--target",
            ARM_TARGET,
            "--bin",
            binary,
            *features,
        )

        elf = target_directory / binary
        binary_path, uf2_path, dfu_path = artifact_paths(layout, half, backlight_curve)
        binary_path.parent.mkdir(parents=True, exist_ok=True)
        run(str(objcopy), "-O", "binary", str(elf), str(binary_path))
        run(
            sys.executable,
            "-B",
            str(ROOT / "tools" / "nocfree_uf2.py"),
            str(binary_path),
            str(uf2_path),
        )
        dfu_path.unlink(missing_ok=True)
        run(
            nrfutil,
            "dfu",
            "genpkg",
            "--application",
            str(binary_path),
            "--dev-type",
            "82",
            "--dfu-ver",
            "0.5",
            "--sd-req",
            "0xFFFE",
            str(dfu_path),
        )
    curve_label = (
        "" if backlight_curve == "perceptual" else " linear backlight comparison"
    )
    print(f"NocFree {layout}{curve_label} release verification passed", flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Build and verify NocFree firmware on Windows, macOS, or Linux"
    )
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--layout", choices=LAYOUTS, default="ANSI")
    selection.add_argument("--all-layouts", action="store_true")
    parser.add_argument("--backlight-curve", choices=BACKLIGHT_CURVES)
    parser.add_argument("--dongle", action="store_true")
    arguments = parser.parse_args()
    if arguments.backlight_curve == "perceptual" and (
        arguments.all_layouts or arguments.layout != "ANSI"
    ):
        parser.error("the perceptual backlight is available only for ANSI")
    if arguments.dongle and (
        arguments.all_layouts
        or arguments.layout != "ANSI"
        or arguments.backlight_curve is not None
    ):
        parser.error("the D1 dongle build is available only for the default ANSI selection")

    for command in ("cargo", "rustc"):
        if shutil.which(command) is None:
            parser.error(f"{command} was not found on PATH")
    nrfutil = shutil.which("adafruit-nrfutil")
    if nrfutil is None:
        parser.error(
            "adafruit-nrfutil was not found; install it with: "
            f"{sys.executable} -m pip install adafruit-nrfutil"
        )

    local_libclang = ROOT / ".tools" / "llvm" / "bin"
    if sys.platform == "win32" and local_libclang.is_dir():
        os.environ.setdefault("LIBCLANG_PATH", str(local_libclang))

    run("cargo", "fmt", "--package", "nocfree-and-rust", "--", "--check")
    host = rust_host()
    objcopy = llvm_objcopy()
    if arguments.dongle:
        build_dongle(host, objcopy, nrfutil)
        run(
            sys.executable,
            "-B",
            "-m",
            "unittest",
            "discover",
            "-s",
            "tools",
            "-p",
            "test_*.py",
        )
        return
    layouts = LAYOUTS if arguments.all_layouts else (arguments.layout,)
    for layout in layouts:
        backlight_curve = arguments.backlight_curve
        if backlight_curve is None:
            backlight_curve = "perceptual" if layout == "ANSI" else "linear"
        build_layout(layout, host, objcopy, nrfutil, backlight_curve)
    run(
        sys.executable,
        "-B",
        "-m",
        "unittest",
        "discover",
        "-s",
        "tools",
        "-p",
        "test_*.py",
    )
    if arguments.all_layouts:
        print("NocFree all-layout release verification passed")


if __name__ == "__main__":
    main()
