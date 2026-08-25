# Layout variants

The firmware selects exactly one physical layout at compile time. ANSI remains
the default and the only layout verified on the maintainer's keyboard.

| Layout | Source module | Left | Right | Scan source | Status |
|---|---|---:|---:|---|---|
| ANSI | `src/keymap/ansi.rs` | 37 | 47 | Original NocFree community ZMK port and local hardware | Stable candidate; locally hardware-tested |
| ISO | `src/keymap/iso.rs` | 38 | 47 | Official updater ISO images | Experimental; matching hardware not tested |
| JIS | `src/keymap/jis.rs` | 37 | 48 | `electricdoc187/NocFree-and-zmk` `jis-custom` scan map | Experimental; Rust build not hardware-tested |
| KR | `src/keymap/kr.rs` | 39 | 50 | Official updater/product data and documented `0x21/P0` extra port | Experimental; matching hardware not tested |

Shared behavior lives in `src/keymap.rs`. Each layout module owns only its
physical counts, visual-to-raw transform, HID usages, Fn positions, and PCA9555
addresses. Scanner, report, NocFree Link persistence, USB product name, and
artifact packaging consume the selected module. Persisted Link keymaps include
record version, layout ID, key count, and CRC, so records from another layout
are rejected instead of being misapplied.

The record format change intentionally invalidates older version-3 NocFree Link
keymap records because they do not identify their physical layout. After the
first version-4 boot, bindings return to the selected layout defaults; BLE host
and split bonds use separate records and are not part of this reset.

## Build one layout

Run one of these from Windows PowerShell:

```powershell
& '.\tools\build-release.ps1' -Layout ANSI
& '.\tools\build-release.ps1' -Layout ISO
& '.\tools\build-release.ps1' -Layout JIS
& '.\tools\build-release.ps1' -Layout KR
```

The command tests and builds both roles. Stable ANSI output is written to
`firmware`. ISO/JIS/KR output is written to `firmware/experimental` and its
filename includes NocFree &, Rust, ZMK-based, layout, Experimental, and Left or
Right. Flash only a matching Left/Right pair from one build.

## Current limitations

- No ISO, JIS, or KR Rust artifact has been tested on matching hardware in this
  repository. Passing compilation and synthetic scan tests is not physical
  validation.
- The JIS source uses tap Muhenkan / hold Fn on the left Eisu key. The initial
  Rust variant keeps that physical key as Fn only; tap/hold behavior is deferred
  until a JIS tester is available. The right Fn key remains Fn.
- Host keyboard layout/IME selection remains an operating-system setting. A
  firmware variant does not install or switch the host layout.
- Never flash an Experimental image onto a different physical layout merely to
  test it; electrical positions and key counts differ.

## Next physical test order

When the ANSI keyboard is available, flash a matching newly built pair (right
first, then left), then verify all 84 keys, both Fn positions, Wired USB,
Windows 11 BLE reconnect, backlight synchronization, `Fn+5`/`Fn+0` recovery,
and one NocFree Link edit plus reboot persistence. Only after this ANSI
regression passes should matching-device volunteers test ISO, JIS, and KR.

## Mapping references

- Original community port: <https://github.com/NocFreeKB/NocFree-and-zmk>
- Hardware-tested JIS ZMK branch: <https://github.com/electricdoc187/NocFree-and-zmk/tree/jis-custom>
- Official product layout reference: <https://www.nocfree.com/products/nocfree-and-reservation>
