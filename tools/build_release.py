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


def llvm_objcopy() -> Path:
    sysroot = Path(output("rustc", "--print", "sysroot"))
    path = sysroot / "lib" / "rustlib" / rust_host() / "bin" / "llvm-objcopy"
    if sys.platform == "win32":
        path = path.with_suffix(".exe")
    if not path.is_file():
        raise RuntimeError(
            "llvm-tools-preview is missing; run: rustup component add llvm-tools-preview"
        )
    return path


def artifact_paths(
    layout: str, half: str, backlight_curve: str = "linear"
) -> tuple[Path, Path, Path]:
    directory = ROOT / "firmware"
    if layout == "ANSI" and backlight_curve == "linear":
        return (
            directory / f"NocFree_Rust_{half}.bin",
            directory / f"NocFree_And_Rust_ZMK_Based_ANSI_{half}.uf2",
            directory / f"NocFree_Rust_{half}_DFU.zip",
        )
    directory /= "experimental"
    if layout == "ANSI":
        stem = f"NocFree_And_Rust_ZMK_Based_ANSI_Perceptual_Backlight_Experimental_{half}"
    else:
        stem = f"NocFree_And_Rust_ZMK_Based_{layout}_Experimental_{half}"
    return (
        directory / f"{stem}.bin",
        directory / f"{stem}.uf2",
        directory / f"{stem}_DFU.zip",
    )


def build_layout(
    layout: str,
    host: str,
    objcopy: Path,
    nrfutil: str,
    backlight_curve: str = "linear",
) -> None:
    selected_features = [f"layout-{layout.lower()}"]
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
    curve_label = "" if backlight_curve == "linear" else " perceptual backlight"
    print(f"NocFree {layout}{curve_label} release verification passed", flush=True)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Build and verify NocFree firmware on Windows, macOS, or Linux"
    )
    selection = parser.add_mutually_exclusive_group()
    selection.add_argument("--layout", choices=LAYOUTS, default="ANSI")
    selection.add_argument("--all-layouts", action="store_true")
    parser.add_argument(
        "--backlight-curve", choices=BACKLIGHT_CURVES, default="linear"
    )
    arguments = parser.parse_args()
    if arguments.backlight_curve != "linear" and (
        arguments.all_layouts or arguments.layout != "ANSI"
    ):
        parser.error("the perceptual backlight comparison is available only for ANSI")

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
    layouts = LAYOUTS if arguments.all_layouts else (arguments.layout,)
    for layout in layouts:
        build_layout(layout, host, objcopy, nrfutil, arguments.backlight_curve)
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
