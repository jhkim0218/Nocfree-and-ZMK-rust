# Repository working rules

1. Reliability comes before new features.
2. Preserve independent DFU and stock-recovery paths for both halves.
3. Never modify the MBR, SoftDevice, bootloader, factory filesystem, protected flash ranges, or UICR without separate verification.
4. Work one phase at a time and do not advance until its acceptance criteria pass.
5. Run automated checks before requesting a hardware test.
6. Give every behavior change an exact physical-test checklist with artifact names and hashes.
7. The left half remains authoritative for keymaps, layers, merged key state, backlight state, HID reports, and output selection.
8. Right-half input must carry enough source metadata to detect loss and establish cross-half order.
9. Cross-half ordering must use sequence numbers and comparable source time, not arrival order alone.
10. Backlight synchronization must use absolute authoritative state, not relative toggles.
11. A reconnected split is not ready until required key and backlight state has been reconciled.
12. Do not start full dongle implementation until split reconnect, backlight convergence, and input ordering pass.
13. Prove dongle recovery before flashing feature firmware to it.
14. ANSI is the only locally hardware-verified layout.
15. ISO, JIS/JP, and KR variants remain Experimental until tested on matching hardware.
16. Never guess an electrical scan mapping.
17. Persisted layout data must include record version, layout identity, key count, and CRC.
18. Firmware filenames must identify the hardware role and layout.
19. Do not treat reported factory-firmware battery life as Rust-firmware power data.
20. Record current real-hardware regressions even when older documentation says Complete.
21. Do not flash or copy firmware to hardware without explicit user authorization for that deployment.
22. After each completed feature, run the relevant automated and physical tests, update documentation, and commit the verified result.
