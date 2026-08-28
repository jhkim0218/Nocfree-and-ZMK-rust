# 工場版互換ロードマップ

[English](ROADMAP.md) · [한국어](ROADMAP_ko.md) · [日本語](ROADMAP_ja.md)

ローカルの NocFree & ANSI V2.3.0 UF2、公式資料、現在の Rust firmware を
比較します。binary size だけでは機能の存在を証明できないため、文書、protocol、
GPIO、実機で観測した動作だけを根拠にします。工場 UF2 から source code を
取り出してコピーすることはできません。

## 現在の優先順位

| 優先 | 項目 | 現在 | 完了条件 |
|---|---|---|---|
| P0 | 独立復旧 | 左右1200-baud、`Fn+5`/`Fn+0`、stock round trip 実績あり | 新 image でも左右を混ぜず復旧可能 |
| P1 | ANSI 回帰 | 過去 image は84キー、USB/BLE、switch、Fn を検証済み | 今回の 5 ms と brightness curve を実機確認 |
| P2 | Battery | 工場版の換算と filter、満充電100%を確認 | 放電曲線、実測電圧、左右消費電流で補正 |
| P3 | Split reconnect | ATT MTU 23、段階 advertising、+8 dBm、暗号化 | 距離/RSSI/時間/電流を統制して安定確認 |
| P4 | 左右入力順序 | timestamp、sequence、clock sync、5 ms queue | 長時間 loss/duplicate/reorder/stuck=0 |
| P5 | Layout | ANSI stable、ISO/JIS/KR Experimental | 各 matching hardware で全キーと復旧を確認 |
| サポート済み | Rust USB dongle / 2.4G | 暗号化 BLE、専用 bond、固定最大サイズ report、切断 release、USB HID は software 検査合格。共通 image は ANSI 実機と idle 後再接続を確認済み。純正 ESB/nRF24/numpad は対象外 | ISO/JIS/KR の対応 keyboard pair で pairing・再接続・順序・latency・復旧・共存を確認する |

## Battery

SAADC から divider と ADC scale で mV を復元し、工場版と同じ段階式で percent に
変換し、前回値3対新値1の filter を使います。`Fn+I` を3秒押すと
`L {左%} R {右%}` を入力します。満充電で左右100%は確認しましたが、放電精度や
省電力を証明するものではありません。

一定間隔で表示%、実測電圧、使用時間、USB/BLE、backlight、再接続回数を左右別に
記録します。100%から10%未満まで複数点を集め、単調性、急落、左右差を確認してから
threshold を変更します。赤 LED は10%以下で0.5秒 ON/OFFですが、十分に放電して
いないため実機未検証です。推測だけで換算表を変更しません。

## Backlight と LED

backlight は左が所有する version 付き絶対状態を右へ送り、toggle、brightness、
timeout、wake、reconnect 後に収束します。30秒無入力で左右が消灯し、最初のキーで
復帰します。`Fn+F5` は暗く、`Fn+F6` は明るくします。PWM 10 kHz はちらつきを
見えなくする周波数で、明るさは duty が決めます。0/20/40/60/80/100% の設定を
体感上分けるため二次 duty curve を導入しました。実機確認までは候補です。

青 pairing と赤 low-battery blink はありますが、charging/full を含む工場版の
全 LED state は未完成です。pin、polarity、観測状態を得てから追加します。

## Cross-half ordering

right event は source timestamp と sequence を持ち、left は接続時3 sample と60秒更新で
clock offset を求めます。両側を同じ 5 ms queue に入れ、L-R-L/R-L-R を保持します。
合成10,000 event では3/4/5 ms が clean でした。以前の3 msは限定実機試験済み、
8 msは余裕候補、現在の5 msは遅延との折衷で実機未検証です。

`src/scanner.rs` の `REORDER_WINDOW_MS` を変更する場合は左右を同じ commit から
buildし、USB/BLE、高速交差入力、再接続直後、長時間 drift を確認します。compile
や unit test だけで安定値にしません。

## Layout と HID

ANSI は37+47キー、ISO 38+47、JIS 37+48、KR 39+50です。JIS の usage
0x87/0x89/0x8a は JIS build だけ HID bitmapを19 byteへ広げ、ANSI/ISO/KR は
従来の16 byteを維持します。debouncer は選択配列の大きい側に合わせます。
software test は matching hardware の証明ではありません。

## 省電力

interrupt scan、250 ms safety scan、30秒 backlight timeout、5分 System OFF が
あります。左 central は左 wake pin または USB で復帰します。right-only の完全な
deep sleep/wake は split reconnect と wake source の制約があり、電流測定が必要です。
既存機能を壊す推測的 sleep は追加しません。

## 次の実機順序

1. matching ANSI pair を右、左の順で更新し、各1200-baud復旧を確認。
2. Wired で84キー、両 Fn、非文字、L-R-L/R-L-R、backlight 6段階を確認。
3. Windows 11 BLE で同じ入力、profile 1/2、Wired往復、自動再接続を確認。
4. Android の既存 bond で入力と profile 切替を確認。
5. 30秒消灯、first-key wake、右再起動後のbacklight収束を確認。
6. `Fn+5`/`Fn+0` が短押しでは発動せず3秒で正しい側だけ DFU になることを確認。
7. 長期 battery と消費電流 log を開始。

ISO/JIS/KR は ANSI 回帰後、該当実機の tester だけが試します。Quick Text、追加の
lighting effect、firmware updater、ZMK Studio は現在の必須範囲ではありません。
