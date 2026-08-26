# NocFree Rust firmware 引き継ぎ

[English](HANDOFF_en.md) · [한국어](HANDOFF.md) · [日本語](HANDOFF_ja.md)

別の maintainer や AI が会話履歴なしで継続するための入口です。現在の worktree、
test、接続 hardware、artifact hash を正とし、source や commit だけで実機への配備を
推測しません。最初に `git status` と `git log -1` を確認し、`README_ja.md`、
`RECOVERY_ja.md`、`ROADMAP_ja.md`、`PROGRESS_ja.md`、`LAYOUTS_ja.md` を読みます。

## Repository と安全

- repository: `https://github.com/jhkim0218/Nocfree-and-ZMK-rust.git`
- original ZMK: `https://github.com/NocFreeKB/NocFree-and-zmk`
- MCU: nRF52833、S140 7.3.0、application `0x27000..0x64fff`
- 左が central/host HID、右は split input を左へ送る
- 左右それぞれ独立した 1200-baud CDC recovery がある
- 明示的承認なしに flash しない。左右・配列・build を混在させない

NocFree & には外部 reset button がありません。未知の pad や架空の shortcut を
使いません。物理 DFU が必要なら user へ一度通知し、その返答を待ちます。

## 現在の構造

両側は共通 scanner で PCA9555 を読み、選択 layout が物理数と変換を提供します。
ANSI が既定かつ唯一の実機検証済み配列で、ISO/JIS/KR は Experimental です。
right は encrypted BLE split で left へ接続し、left が USB/BLE HID、3つの persistent
bond、NocFree Link、battery text、switch routing、DFU command を担当します。

right snapshot は source time、sequence、reconcile を持ち、left は時計差を求めて
両側を同じ queue で並べます。現在の `REORDER_WINDOW_MS` は 5 ms 候補です。過去の
3 ms は限定的な Wired/Windows 11 BLE 実機試験済みですが、5 ms は未検証です。
調整時は左右を同じ build で更新し、USB/BLE の高速 L-R-L/R-L-R を確認します。

backlight は left 所有の version 付き絶対状態です。`Fn+F5` が down、`Fn+F6` が up、
PWM は左右とも 10 kHz です。線形 duty の上側が同じに見えたため、
0/20/40/60/80/100% を体感上分ける二次 curve に変更しました。30秒 timeout と
first-key wake は維持されますが、新 curve は ANSI 実機確認が必要です。

## OS、shortcut、switch

初期値は **Windows mode** です。`Fn+M` 1秒で macOS、`Fn+N` 1秒で Windows を保存し、
短押しは M/N を入力します。`Fn+1/2/3` は短押しで profile 選択、長押しで pairing、
`Fn+I` 3秒で左右 battery、`Fn+5` 3秒で左 UF2、`Fn+0` 3秒で右 UF2、`Fn+Esc` は
左 restart です。左 switch は Wired / 未実装2.4G安全無出力 / BLE、右は power です。
BLE host 実機試験は Windows 11 と Android のみです。

## Portable build

Windows、macOS、Linux で ARM target、LLVM tools、`adafruit-nrfutil` を用意し、
`python3 -B tools/build_release.py --all-layouts` を実行します。Python builder は
standard library のみで、format、native test、host/ARM Clippy、左右 build、
BIN/UF2/DFU package、contract test を行います。PowerShell script は wrapper です。

## 残作業

1. 全4 layout software pipeline と新 artifact hash を記録。
2. ANSI 実機で5 ms、6 brightness、USB/BLE、全キー、Fn、switch、sleep/wake、DFUを確認。
3. split jitter、長時間 loss/duplicate/reorder、距離、再接続、+8 dBm 電力を測定。
4. battery の実測電圧・電流・完全放電 log を得てから threshold を変更。
5. charging/full など LED state は evidence に基づいて追加。
6. ISO/JIS/KR は matching hardware だけで試験。
7. factory dongle/2.4 GHz は未実装。Quick Text、追加 effect、updater、ZMK Studio は必須外。

flash boundary、左右独立 recovery、left authoritative state、ANSI HID shape を必ず
維持します。関連 test を省略して成功と報告せず、機能ごとに test、docs、artifact、
commit の順で完了し、push は user が依頼したときだけ行います。
