# NocFree Rust 進捗

[English](PROGRESS.md) · [한국어](PROGRESS_ko.md) · [日本語](PROGRESS_ja.md)

長い実験記録の現在判定を日本語でまとめます。詳細な時刻、RSSI、過去 hash は
[PROGRESS.md](PROGRESS.md)、優先順位は [ROADMAP_ja.md](ROADMAP_ja.md)、復旧は
[RECOVERY_ja.md](RECOVERY_ja.md) を基準にします。

## 検証 baseline

- repository: `https://github.com/jhkim0218/Nocfree-and-ZMK-rust.git`
- original: `https://github.com/NocFreeKB/NocFree-and-zmk`
- default は ANSI 37+47。ISO/JIS/KR は Experimental・実機未検証
- 左だけが USB/BLE host HID、右は encrypted split input
- app `0x27000..0x64fff`、左右独立1200-baud recovery と factory round trip を維持
- BLE host 実機範囲は Windows 11 と Android のみ

## 完了済みの実機範囲

過去の matching ANSI image では USB、BLE、全84キー、両 Fn、非文字、物理
Wired/Bluetooth/2.4G position、右 power switch、BLE profile 1/2、Windows と Android
の multi-pairing、左右 DFU、factory V2.3.0 round trip、NocFree Link の変更と再起動後
保存を確認しました。右 USB は HID ではなく独立 CDC recovery で正常です。

battery は factory V2.3.0 の divider/SAADC/段階式と3:1 filterを再現し、満充電で
`L 100 R 100` を確認しました。完全放電と消費電流は未検証です。青 pairing LED は
long-hold 後に継続し、bonded profile 選択で停止しました。赤10%以下は未検証です。

backlight の絶対状態同期、右再起動後の収束、10回 toggle、右 key wake、30秒両側消灯、
USB 電源に依存しない timeout は過去 image で通りました。

## Dongle D0 recovery

2026-08-27 の factory dongle baseline は `NocFree_Dongle`、
`VID_2886&PID_8029`、serial `E19D2CEA0B437049`、COM6、HID `MI_02` でした。
1200-baud touch で `VID_239A&PID_002A`、`NocFree &`、`nRF Serial`、COM14 の
serial DFU に入り、official V2.3.1 application-only package 転送後に元の USB
identity と interface が戻りました。Rust dongle feature firmware は未導入です。
左右が Rust のため factory ESB input はこの D0 では未確認です。

## ANSI Rust 2.4G dongle — 実機検証済み

ANSI の左右と dongle は custom BLE link で接続し、keyboard/consumer HID report を
送信します。dongle 再接続、高速な左右入力と modifier、BLE→2.4G 復帰、各 device の
1200-baud recovery round trip を確認しました。ISO/JIS/KR dongle image は build 済み
ですが、対応 layout の実機検証は未完了です。

## Split P3

初期 reconnect failure は不要な ATT MTU exchange で発生しました。P3.1 は標準 MTU 23
にしてその段階を除去しました。約30cmで power cycle 後55.431秒と17.736秒に入力が
戻り、左 recovery 後は4.358秒で split-ready でした。

P3.2 は未接続 right advertising を最初10秒250 ms、次の50秒500 ms、その後1秒とし、
未接続 key で250 msへ戻します。段階、key reset、Wired/BLE input、right recovery は
通りましたが、低 RSSI の security timeout と体感 delay が残ります。

P3.3 は nRF52833 の最大 +8 dBm を split に適用しました。入力は通りましたが、距離、
reconnect、security、消費電流を同条件で比較していないため最適化完了ではありません。

## 左右入力順序 P4

`jam -> ajm` や `삼 -> ㅅ마` の高速交差入力に対し、source timestamp、right sequence、
split clock sync、reconcile、単一 global queue を実装しました。1〜5 ms・10,000 synthetic
event では1/2 msが失敗、3/4/5 msが clean でした。以前の3 ms imageは Wiredの連続
`jam` と Windows 11 BLE input を通過しました。

その後8 msの余裕候補を作り、現在は latency と jitter margin の折衷として 5 ms です。
`REORDER_WINDOW_MS=5` は automated test のみで実機未検証です。runtime queue 全体、
long drift、reconnect edge、Android で loss/duplicate/reorder/stuck=0 が必要です。

## 現在の software candidate

- `Fn+F5` は brightness down、`Fn+F6` は up を testで固定
- 左右 PWM は共通 10 kHz
- 0/20/40/60/80/100% を体感上分離する二次 duty curve。実機確認が必要
- JIS usage 0x87/0x89/0x8a のみ19-byte HID、ANSI/ISO/KR は16-byte維持
- JIS右48、KR右50を扱える selected-layout max debouncer
- Windows/macOS/Linux 共通 `tools/build_release.py`、PowerShellは wrapper

## 次の判定順序

1. 4 layout の host/ARM Clippy、左右 release、UF2/DFU、contractを全通過。
2. 新 ANSI pair を右、左の順で導入し、各側の recovery を確認。
3. Wired で84キー、両 Fn、`jam`/`삼`、6 brightness、timeout/wake。
4. Windows 11 BLE で profile、Wired往復、自動 reconnect、同じ入力。
5. Android の既存 bond で入力。
6. `Fn+5`/`Fn+0` は短押し無動作、3秒で正しい側だけ DFU。
7. その後に docs、artifact hash、commit/push を確定。

Quick Text、追加 lighting effect、power UI、firmware updater、ZMK Studio は必須外です。
ANSI USB dongle の Rust wireless input path は実機検証まで完了しました。
