# DFU と役割別の工場版復旧

[English](RECOVERY.md) · [한국어](RECOVERY_ko.md) · [日本語](RECOVERY_ja.md)

NocFree & には外部 reset button がありません。未確認の PCB pad を短絡したり、
電源の抜き差しを reset-pin double tap とみなしてはいけません。Rust firmware は
Fn shortcut と別に、左右と Experimental ドングルそれぞれの 1200-baud CDC 復旧経路を維持します。

## 保護範囲

`0x00000..0x26fff` の MBR/S140、`0x6d000..0x73fff` の工場 filesystem、
`0x74000..0x7ffff` の UF2 bootloader は保護します。Rust application が書けるのは
`0x27000..0x64fff` です。`0x65000..0x67fff` は BLE profile、`0x68000` は予約、
`0x69000` はドングル bond、`0x6a000..0x6cfff` は設定、split bond、
Link keymap の保存領域です。build と UF2 round-trip test は `0x65000` を越える
image を拒否します。Left、Right、Dongle を役割と配列に一致させてください。

## DFU への入り方

- 左 Rust CDC を 1200 baud で touch: 左独立 UF2
- 右 Rust CDC を 1200 baud で touch: 右独立 UF2
- ドングル Rust CDC を 1200 baud で touch: ドングル独立 UF2
- ドングル Rust CDC を 2400 baud で touch: bond 消去後に再起動
- `Fn+5` を3秒: 左 UF2
- `Fn+0` を3秒: split が動作中なら右 UF2
- `Fn+Esc`: 左 application の再起動であり DFU ではない
- 未処理 panic: GPREGRET `0x57` を保存して UF2 へ再起動

左を 1200-baud 復旧するときは物理 switch を中央の **Wired** にします。port が
すぐ消え、open/close が missing-device error になっても reset 済みの場合があるため、
再試行前に UF2 volume と bootloader CDC を確認します。

Windows の COM 番号は固定ではありません。parent instance ID の
`RUST-LEFT`/`RUST-RIGHT`/`RUST-DONGLE`（ドングル parent は
`USB\VID_239A&PID_80D8\RUST-DONGLE`）、UF2 の `VID_239A&PID_0029`、serial DFU の
`VID_239A&PID_002A` と chip serial で役割を特定します。macOS は
`stty -f /dev/cu.usbmodemXXXX 1200`、Linux は `stty -F /dev/ttyACM0 1200` が
基本形ですが、実際の device と左右を先に確認してください。

## UF2 と serial DFU

`INFO_UF2.TXT` が NocFree &、S140 7.3.0、NocFree & Board-ID を示すことを確認します。
左右の volume 名は同じなので、消えた CDC と反対側 application の存在も確認します。
左は `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`、右は
`firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2` のみをコピーします。
共通 dongle は `firmware/NocFree_And_Rust_ZMK_Based_Dongle.uf2` を使います。
サポートされる共通 Rust 2.4G dongle の bootloader と app 復旧は ANSI 実機確認済みです。ISO/JIS/KR は対応
キーボードで検証してください。

serial DFU は確認済み `PID_002A` port だけで実行します。`adafruit-nrfutil` は
過去に `No data received on serial port` と表示しながら exit 0 を返したため、
失敗表示がないこと、`Device programmed.`、目標 Rust parent の復帰をすべて
成功条件にします。

## 検証済み工場版 round trip

2026-08-21 に左右とも、対象 parent/CDC の確認、対象だけを1200 touch、UF2 の確認、
役割別 factory V2.3.0 image の hash と copy、factory parent/chip serial、factory CDC
の1200 touch、同じ chip の `PID_002A`、役割別 Rust DFU ZIP、最後の Rust parent と
実入力までを完了しました。

factory 左 SHA-256 は
`A3FF612B94E9CE0C12BEFD9FF19ECDE5D6E4DB964C0345E432B71ED7A2C5BC91`、右は
`E1F851906B3E35117B8A8AAC09E5C8273D75921F64D1D3B1496E60A53D3E1C66` です。

## 検証済み dongle serial-DFU round trip

2026-08-27 に factory dongle の application-only recovery を確認しました。
application baseline は `NocFree_Dongle`、`VID_2886&PID_8029`、serial
`E19D2CEA0B437049`、CDC `MI_00`、HID `MI_02` です。application CDC を1200 baudで
open/closeすると UF2 disk ではなく、`VID_239A&PID_002A`、product `NocFree &`、
`nRF Serial` の serial-only bootloader に入ります。COM6からCOM14へ変わったため、
COM番号を固定せず serial と VID/PID を確認します。

使用した repository 外の official V2.3.1 application-only package
`D:\study\nocfree\official_v2_3_1\dongle.zip` の SHA-256 は
`02C1EE2BB420E374E51AC6B0C0EE7A422796DFDFF0AC2707CA28096A564C0567` です。
`softdevice_req=0x0123` で、bootloader、SoftDevice、UICR を含みません。user承認後の
115200-baud転送は `Device programmed.` で完了し、元のUSB identityとinterfaceが
すべて戻りました。

左右が Rust firmware のため factory ESB key input は未確認です。この結果が証明する
のは dongle recovery と USB identity であり、factory radio compatibility ではありません。

予期しない側が消える、役割・配列・hash が違う、目標 parent が戻らない、key が
押されたまま、発熱、app と bootloader の両方が見えない場合は直ちに停止します。
その後は反対側に触れません。自動 build は実機 flash の許可ではなく、必ず user の
明示的な承認が必要です。
