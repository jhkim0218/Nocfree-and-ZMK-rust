# NocFree-and-rust

[English](README.md) · [한국어](README_ko.md) · [日本語](README_ja.md)

> [!CAUTION]
> `develop` ブランチには開発中の作業が含まれます。ファームウェア成果物は
> 自動検査に合格していますが、**実機では検証されていません**。

> [!NOTE]
> 左右入力の順序待ち時間は、実機検証済みの **3 ms** から **8 ms** 候補を経て、
> 現在の折衷値 **5 ms** へ変更しました。ビルダーは `src/scanner.rs` の
> `REORDER_WINDOW_MS` を調整できます。変更後は左右を一緒に再ビルド・
> フラッシュし、USB と Bluetooth の両方で高速な交互入力を確認してください。

NocFree & ANSI キーボードのコミュニティ ZMK 実装を、nRF52833 向けの
`no_std` Rust ファームウェアへ移植するプロジェクトです。原典は
[`NocFreeKB/NocFree-and-zmk`](https://github.com/NocFreeKB/NocFree-and-zmk)
であり、本リポジトリは独立した Rust 移植版です。

> [!IMPORTANT]
> ファームウェアの**既定は Windows モード**です。macOS 用のキー動作へ
> 切り替えるには `Fn+M` を1秒間長押しします。設定は再起動後も保存されます。
> Windows に戻すには `Fn+N` を1秒間長押しします。短押しは通常どおり M/N を
> 入力します。

実機検証済みの物理配列は84キー ANSI のみです。ISO、JIS、KR は個別に
ビルドできますが、**Experimental・実機未検証**です。使用前に
[LAYOUTS_ja.md](LAYOUTS_ja.md) を読んでください。

| 配列 | 状態 | 実機検証 |
|---|---|---|
| ANSI | 5 ms 入力順序候補 | 以前の 3 ms は検証済み。現在の 5 ms は未検証 |
| ISO | Experimental | ISO 実機では未検証 |
| JIS | Experimental | JIS 実機では未検証 |
| KR | Experimental | KR 実機では未検証 |

## 役割

- **左 (`central`)**: 左側キー、右側 split 入力、全キーマップ、USB/BLE HID、
  3つの BLE マルチペアリングスロット、NocFree Link を担当します。
- **右 (`right`)**: 右側キーを読み、暗号化した BLE split で左へ送ります。
  右 USB は HID ではなく、独立した CDC 復旧インターフェースです。

両側は PCA9555 を SDA P0.11/SCL P1.09、100 kHz で読みます。入力は
active-low、debounce は 5 ms、BLE PHY は 1M です。TWIM 初期化前に最大9回の
bus-clear clock と STOP を送るため、再起動中に I2C が停止しても回復できます。

## Windows・macOS・Linux でビルド

共通の Python 標準ライブラリ製ビルダー `tools/build_release.py` を使用します。
Rust、Python 3、C/C++ toolchain と libclang を用意してから次を実行します。

```text
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
python3 -m pip install adafruit-nrfutil==0.5.3.post16
python3 -B tools/build_release.py --layout ANSI
```

Windows でコマンド名が `python` の場合は `python3` を置き換えてください。
macOS で Xcode tools から libclang を検出できない場合は Homebrew LLVM、
Debian/Ubuntu では `clang` と `libclang-dev` を導入します。それでも見つからない
場合は libclang shared library の directory を `LIBCLANG_PATH` に指定します。
Experimental 配列は `--layout ISO`、`--layout JIS`、`--layout KR`、全配列の
検査は次のコマンドです。

```text
python3 -B tools/build_release.py --all-layouts
```

Windows では従来の薄い PowerShell ラッパーも使用できます。

```powershell
& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1' -Layout ANSI
```

ビルダーは format、実行 OS の host test、host/ARM Clippy、左右の release
build、BIN/UF2/serial-DFU ZIP 作成、ベクタ・アドレス・family・round-trip・
ZIP 内容を検査します。ANSI は `firmware`、実験配列は
`firmware/experimental` に出力されます。

## フラッシュ前の注意

- 左には `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2`、右には
  `firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2` だけを使用します。
- 異なる配列、異なるビルド、左右を混在させないでください。
- UF2 はアプリ領域 `0x27000..0x64fff` だけを書き込み、SoftDevice、保存領域、
  工場ファイルシステム、UF2 bootloader を保護します。
- 先に [RECOVERY_ja.md](RECOVERY_ja.md) を読み、左右独立の 1200-baud 復旧経路を
  確保してください。

キーボード上で動くコードはすべて Rust `no_std` です。Python は PC 上で
成果物を作成・検査するだけで、キーボードには書き込まれません。

## 左右入力順序

右 snapshot は source time、sequence、reconcile 情報を送ります。左は接続時に
時計差を推定し、ローカルとリモートのイベントを同じ 5 ms queue で並べます。
1〜5 ms・10,000イベントの合成試験では 3/4/5 ms が正常でした。以前 8 ms の
候補も作られましたが、現在は遅延と jitter 余裕の折衷として 5 ms です。

ビルダーが調整する場所は `src/scanner.rs` の `REORDER_WINDOW_MS` です。小さく
すると遅延が減り、大きくすると BLE transport jitter への余裕が増えます。
変更後は左右を同時に再ビルド・フラッシュし、USB と BLE の両方で高速な
L-R-L/R-L-R 入力を試してください。コンパイル成功だけでは値を決められません。

## バックライト

`Fn+F5` は暗く、`Fn+F6` は明るくする正しい向きです。設定は
0/20/40/60/80/100% の6段階で左右へ絶対状態として同期されます。PWM は
10 kHz のままです。周波数はちらつきを見えなくし、明るさは duty が決めます。
単純な線形 duty では上側が同じに見えたため、体感に合わせた二次曲線を使って
各段階を離しました。この新しい曲線は ANSI 実機で最終確認が必要です。

30秒間 NocFree のキー入力がなければ左右が消灯し、最初のキーで両方が復帰します。
手動 OFF、timeout、再接続、右側再起動後も左が所有する絶対状態へ収束します。

## 主なショートカット

| キー | 操作 |
|---|---|
| `Fn+1` / `Fn+2` / `Fn+3` 短押し | BLE profile 1/2/3 を選択 |
| `Fn+1` / `Fn+2` / `Fn+3` 長押し | 選択 profile を pairing mode にする |
| `Fn+Tab` | バックライト ON/OFF |
| `Fn+F5` / `Fn+F6` | 明るさを20%下げる / 上げる |
| `Fn+I` 3秒 | `L {左電池%} R {右電池%}` を入力 |
| `Fn+M` 1秒 | macOS mode を保存 |
| `Fn+N` 1秒 | Windows mode を保存 |
| `Fn+5` 3秒 | 左を UF2 bootloader へ再起動 |
| `Fn+0` 3秒 | split が正常なら右を UF2 bootloader へ再起動 |
| `Fn+Esc` | 左アプリを再起動（DFU ではない） |

## BLE と物理スイッチ

左スイッチの Wired は USB HID、Bluetooth は選択中の bond slot、2.4G は未実装の
ため安全な無出力です。右スイッチは上が OFF、下が ON です。BLE profile は
「3台を同時に送信」する機能ではなく、3つの bond を保存して1つを選ぶ
multi-pairing です。実機の host 試験は Windows 11 と Android だけで、macOS、
iOS、Linux、3台目 host は未検証です。

## 現在の未完了項目

- 5 ms 入力順序と新しいバックライト曲線の ANSI 実機回帰試験
- ISO/JIS/KR の対応実機試験。JIS の拡張 HID usage はコードと test で対応済み
- 実放電サイクル、消費電流、長期的な battery 最適化
- 充電/満充電など工場版と同等の LED 状態
- 工場 USB dongle / 2.4 GHz receiver mode
- 長距離・低 RSSI・再接続直後の controlled split stress

履歴と次の順序は [PROGRESS_ja.md](PROGRESS_ja.md)、優先順位は
[ROADMAP_ja.md](ROADMAP_ja.md)、作業継続情報は [HANDOFF_ja.md](HANDOFF_ja.md) を
参照してください。
