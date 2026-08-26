# NocFree Rust firmware handoff

[English](HANDOFF_en.md) · [한국어](HANDOFF.md) · [日本語](HANDOFF_ja.md)

This is the continuation entry point for another maintainer or AI. Treat the
current worktree, tests, connected hardware, and artifact hashes as authority;
do not infer deployment from source or commit history. Start with `git status`
and `git log -1`, then read `README.md`, `RECOVERY.md`, `ROADMAP.md`,
`PROGRESS.md`, and `LAYOUTS.md`.

## Repository and safety

- Repository: `https://github.com/jhkim0218/Nocfree-and-ZMK-rust.git`
- Original community ZMK: `https://github.com/NocFreeKB/NocFree-and-zmk`
- MCU: nRF52833, S140 7.3.0, application `0x27000..0x64fff`
- Left is central and the only host HID. Right sends split input to left.
- Each half has an independent 1200-baud CDC recovery path.
- Never flash hardware without explicit user approval. Never mix halves,
  layouts, or artifacts from different builds. Verify role before every copy.

NocFree & has no external reset button. Do not invent a reset shortcut or short
unknown pads. If physical DFU is required, notify the user once and wait.

## Current architecture

Both halves scan PCA9555 expanders through shared `hardware_scanner.rs`. The
selected layout supplies physical counts and transforms. ANSI is the default
and only hardware-verified layout; ISO, JIS, and KR are Experimental. The right
connects to the left through encrypted BLE split. The left provides USB HID,
BLE HID with three persistent host bonds, NocFree Link, battery text output,
switch routing, and role-specific DFU commands.

Cross-half snapshots carry source time, sequence, and reconcile metadata. The
left estimates the remote clock and orders local/remote events in one queue.
The current `REORDER_WINDOW_MS` candidate is 5 ms. The older 3 ms value passed
limited Wired and Windows 11 BLE hardware stress; the current value still needs
hardware validation. When tuning, rebuild and flash both halves together and
test fast L-R-L and R-L-R over USB and BLE.

Backlight state is left-owned, versioned, absolute, and resent after reconnect.
`Fn+F5` is down and `Fn+F6` is up. Both halves use a 10 kHz PWM. Six settings
0/20/40/60/80/100% now use a perceptual quadratic duty curve because linear
electrical steps looked almost identical near the top. The curve requires an
ANSI hardware check. Thirty-second auto-off and first-key wake remain intact.

## OS and shortcuts

The persisted system setting starts in **Windows mode**. Hold `Fn+M` for one
second for macOS and `Fn+N` for one second for Windows; short presses type M/N.
Profile 1/2/3 short presses select bonds and long presses enter pairing.
`Fn+I` held three seconds types left/right battery percentages. `Fn+5` held
three seconds boots left UF2; `Fn+0` held three seconds boots right UF2 while
split is working. `Fn+Esc` restarts left and is not DFU.

The left physical switch selects Wired, unimplemented 2.4G safe-off, or BLE.
The right switch is physical power. BLE host hardware testing was performed
only on Windows 11 and Android. It is multi-pairing, not simultaneous output.

## Portable build

Install the ARM target, LLVM tools, and `adafruit-nrfutil`, then run on Windows,
macOS, or Linux:

```text
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
python3 -m pip install adafruit-nrfutil==0.5.3.post16
python3 -B tools/build_release.py --all-layouts
```

The Python builder is standard-library-only. It performs formatting, native
tests, host/ARM Clippy, both role builds, BIN/UF2/DFU packaging, then all Python
contracts. `tools/build-release.ps1` is only a Windows compatibility wrapper.
Use `python` in place of `python3` if appropriate.

## Known open work

1. Run the full all-layout software pipeline and record fresh artifact hashes.
2. On ANSI hardware, verify 5 ms ordering, six brightness levels, both output
   modes, all keys, both Fn keys, switches, timeout/wake, reconnect, and DFU.
3. Measure real split jitter, long-run loss/duplicates/reordering, distance,
   reconnect time, and +8 dBm power cost.
4. Log a full battery discharge with measured voltage and current before
   changing thresholds or claiming power optimization.
5. Complete charging/full and other factory LED states only from evidence.
6. Test ISO/JIS/KR only on matching physical keyboards. JIS extended usages
   and larger right scan count are software-covered but not hardware-proven.
7. Factory dongle/2.4 GHz remains unimplemented. Quick Text, extra lighting
   effects, updater compatibility, and ZMK Studio are not required scope.

## Regression invariants

- Preserve the application and factory flash boundaries.
- Preserve both independent 1200-baud paths and the stock round trip.
- Keep left authoritative for host output, bond selection, and backlight state.
- Keep ANSI report shape and behavior unchanged when adding layout-specific HID.
- Never claim tests passed if any relevant command was skipped or failed.
- Test, update documentation and artifacts, then commit a completed feature.
- Do not push unless the user requests it.
