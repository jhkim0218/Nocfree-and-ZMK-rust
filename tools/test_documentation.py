import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DOCUMENT_FAMILIES = {
    "README": ("README.md", "README_ko.md", "README_ja.md"),
    "ROADMAP": ("ROADMAP.md", "ROADMAP_ko.md", "ROADMAP_ja.md"),
    "RECOVERY": ("RECOVERY.md", "RECOVERY_ko.md", "RECOVERY_ja.md"),
    "LAYOUTS": ("LAYOUTS.md", "LAYOUTS_ko.md", "LAYOUTS_ja.md"),
    "HANDOFF": ("HANDOFF_en.md", "HANDOFF.md", "HANDOFF_ja.md"),
    "PROGRESS": ("PROGRESS.md", "PROGRESS_ko.md", "PROGRESS_ja.md"),
    "NRF_SOFTDEVICE_PATCH": (
        "vendor/nrf-softdevice/README.nocfree.md",
        "vendor/nrf-softdevice/README.nocfree_ko.md",
        "vendor/nrf-softdevice/README.nocfree_ja.md",
    ),
    "NRF_SOFTDEVICE_MACRO_PATCH": (
        "vendor/nrf-softdevice-macro/README.nocfree.md",
        "vendor/nrf-softdevice-macro/README.nocfree_ko.md",
        "vendor/nrf-softdevice-macro/README.nocfree_ja.md",
    ),
}


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class DocumentationTests(unittest.TestCase):
    def test_ordering_window_contract(self) -> None:
        scanner = read("src/scanner.rs")
        self.assertIn("pub const REORDER_WINDOW_MS: u64 = 5;", scanner)
        self.assertIn("Builder tuning point", scanner)
        for path in DOCUMENT_FAMILIES["README"]:
            document = read(path)
            self.assertIn("5 ms", document)
            self.assertIn("REORDER_WINDOW_MS", document)

    def test_backlight_contract(self) -> None:
        backlight = read("src/backlight.rs")
        keymap = read("src/keymap.rs")
        self.assertIn("BACKLIGHT_PWM_HZ: u32 = 10_000", backlight)
        self.assertIn("percent * percent / 10_000", backlight)
        self.assertIn("Action::Key(0x3e) => Action::BacklightDown", keymap)
        self.assertIn("Action::Key(0x3f) => Action::BacklightUp", keymap)
        for binary in ("src/bin/central.rs", "src/bin/right.rs"):
            self.assertIn("pwm.set_period(BACKLIGHT_PWM_HZ)", read(binary))

    def test_portable_build_contract(self) -> None:
        builder = read("tools/build_release.py")
        compile(builder, "tools/build_release.py", "exec")
        self.assertIn('ARM_TARGET = "thumbv7em-none-eabihf"', builder)
        self.assertIn("sys.platform == \"win32\"", builder)
        self.assertIn("--all-layouts", builder)
        self.assertIn("build_release.py", read("tools/build-release.ps1"))
        self.assertNotIn("LIBCLANG_PATH", read(".cargo/config.toml"))
        for path in DOCUMENT_FAMILIES["README"]:
            document = read(path)
            self.assertIn("build_release.py", document)
            self.assertIn("macOS", document)
            self.assertIn("Linux", document)

    def test_language_coverage(self) -> None:
        for family, paths in DOCUMENT_FAMILIES.items():
            with self.subTest(family=family):
                for path in paths:
                    document = read(path)
                    minimum = 200 if "vendor/" in path else 500
                    self.assertGreater(len(document), minimum, path)
                    for peer in paths:
                        if peer != path:
                            link = Path(peer).name
                            self.assertIn(link, document, f"{path} does not link to {peer}")

    def test_default_os_guidance(self) -> None:
        required = {
            "README.md": ("Windows mode by default", "Fn+M", "Fn+N"),
            "README_ko.md": ("기본값은 Windows 모드", "Fn+M", "Fn+N"),
            "README_ja.md": ("既定は Windows モード", "Fn+M", "Fn+N"),
        }
        for path, phrases in required.items():
            document = read(path)
            for phrase in phrases:
                self.assertIn(phrase, document)

    def test_hardware_validation_disclosure(self) -> None:
        required = {
            "README.md": ("current 5 ms build is not yet hardware-tested", "new curve still needs hardware confirmation"),
            "README_ko.md": ("현재 5 ms 빌드는 실기 미검증", "새 곡선은 실기 확인이 남음"),
            "README_ja.md": ("現在の 5 ms は未検証", "新しい曲線は ANSI 実機で最終確認が必要"),
        }
        for path, phrases in required.items():
            document = read(path)
            for phrase in phrases:
                self.assertIn(phrase, document)

    def test_expanded_layout_input_contract(self) -> None:
        scanner = read("src/hardware_scanner.rs")
        report = read("src/report.rs")
        descriptor = read("src/usb_descriptor.rs")
        self.assertIn("Debouncer::<MAX_HALF_KEY_COUNT>", scanner)
        self.assertIn('feature = "layout-jis"', report)
        self.assertIn("pub const LAST_BITMAP_USAGE: u8 = 0x8a", report)
        self.assertIn("LAST_BITMAP_USAGE", descriptor)
        self.assertIn("KEY_BITMAP_BITS", descriptor)


if __name__ == "__main__":
    unittest.main()
