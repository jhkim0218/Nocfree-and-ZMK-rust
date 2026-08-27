# NocFree-and-rust

[English](README.md) · [한국어](README_ko.md)

> [!CAUTION]
> `develop` ブランチには開発中の作業が含まれます。ファームウェア成果物は
> 自動検査に合格していますが、**実機では検証されていません**。

nRF52833 ベースの NocFree & キーボード向け独立 `no_std` Rust
ファームウェアです。原典
[`NocFreeKB/NocFree-and-zmk`](https://github.com/NocFreeKB/NocFree-and-zmk)
の動作を移植したもので、NocFree 公式ファームウェアではありません。

> [!IMPORTANT]
> - ファームウェアの**既定は Windows モード**です。macOS モードは `Fn+M`、
>   Windows へ戻す場合は `Fn+N` をそれぞれ1秒間長押しします。
> - キーボードの左右と配列に一致するファイルだけをフラッシュしてください。
>   異なるビルドや配列の左右ファイルを混在させないでください。
> - NocFree & には外部リセットボタンがありません。フラッシュ前に
>   [RECOVERY_ja.md](RECOVERY_ja.md) を読み、左右の DFU と純正 V2.3.0 への
>   復元手順を確認してください。
> - 純正 USB ドングル/2.4 GHz 入力は未実装です。左スイッチの 2.4G 位置は
>   意図的に無出力になります。

| 配列 | 現在の状態 | 実機検証 |
|---|---|---|
| ANSI | 既定ビルド、5 ms 入力順序・linear バックライト候補 | 以前の 3 ms は検証済み。現在の 5 ms は未検証 |
| ISO | Experimental | 対応実機では未検証 |
| JIS | Experimental | 対応実機では未検証 |
| KR | Experimental | 対応実機では未検証 |

## 最初に確認すること

キーボードは左右で役割が固定された分割構成です。

- **左 (`central`)** は37キーと右入力をまとめ、全キーマップを処理して USB
  または Bluetooth HID を出力します。
- **右 (`right`)** は47キーを読み、暗号化 BLE split で左へ送ります。右 USB
  は復旧・診断用であり、キーボード HID は出力しません。

左の物理スイッチで出力を選択します。

| 位置 | モード | 動作 |
|---|---|---|
| 上 | 2.4G | 無出力。純正ドングル通信は未実装 |
| 中央 | Wired | 左 USB ポートから USB HID を出力 |
| 下 | Bluetooth | 左側から Bluetooth HID を出力 |

右スイッチはバッテリー電源を物理的に制御します。上が OFF、下が ON です。
USB 電源はスイッチを迂回するため、USB 接続中は OFF でも右基板が動作します。
左 USB のない Wired モードは HID 出力がないだけで、電源 OFF にはなりません。

初回は次の順序を推奨します。

1. [RECOVERY_ja.md](RECOVERY_ja.md) を読み、左右のファームウェアを確認します。
2. 同じビルドの UF2 ペアをビルドまたはダウンロードします。
3. 片側ずつフラッシュし、まず Wired USB で左右入力を確認します。
4. Bluetooth、物理スイッチ、DFU ショートカット、高速な左右交互入力を
   確認します。

## ビルドとファームウェア

Rust、Python 3、libclang を含む C/C++ ツール、パッケージツールを導入します。

```text
rustup target add thumbv7em-none-eabihf
rustup component add llvm-tools-preview
python3 -m pip install adafruit-nrfutil==0.5.3.post16
```

Windows では環境により `python3` の代わりに `python` を使用できます。macOS
で libclang が見つからない場合は Homebrew LLVM、Debian/Ubuntu では `clang`
と `libclang-dev` を導入します。自動検出に失敗した場合だけ
`LIBCLANG_PATH` を設定してください。

Windows、macOS、Linux で ANSI をビルド・検証します。

```text
python3 -B tools/build_release.py --layout ANSI
```

Experimental 配列は `ISO`、`JIS`、`KR` を指定し、4配列すべては次で処理します。

```text
python3 -B tools/build_release.py --all-layouts
```

Windows PowerShell ラッパーも利用できます。

```powershell
& pwsh -NoProfile -ExecutionPolicy Bypass -File '.\tools\build-release.ps1' -Layout ANSI
```

ビルダーは整形、host/ARM 検査、左右ビルド、BIN/UF2/serial-DFU 生成、アドレスと
内容の検証を行います。ANSI は `firmware`、Experimental 配列は
`firmware/experimental` に出力されます。

リポジトリ収録の ANSI UF2:

- [左/central UF2](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Left.uf2)
- [右/peripheral UF2](firmware/NocFree_And_Rust_ZMK_Based_ANSI_Right.uf2)

上記の既定ファイルは見やすい linear 20% バックライト段階を使います。知覚補正の
比較版は次のようにビルドします。

```text
python3 -B tools/build_release.py --layout ANSI --backlight-curve perceptual
```

再現可能な B ペアは別ファイルです。

- [Perceptual 左 UF2](firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Perceptual_Backlight_Experimental_Left.uf2)
- [Perceptual 右 UF2](firmware/experimental/NocFree_And_Rust_ZMK_Based_ANSI_Perceptual_Backlight_Experimental_Right.uf2)

A/B の両方とも実機未検証です。まず linear の左右ペア、次に perceptual の
左右ペアを試してください。左右で異なる曲線を混在させないでください。

キーボード上で動くコードはすべて Rust `no_std` です。Python と
`adafruit-nrfutil` は PC 上の成果物作成だけに使い、キーボードには入りません。

## 未実装・未検証項目

| 分野 | 残作業 |
|---|---|
| 現在の ANSI 候補 | 5 ms 入力順序、最新の共通 PCA9555 scan、linear/perceptual バックライト両ペアの実機回帰確認 |
| ISO/JIS/KR | ソフトウェアビルドは合格。各配列の実機確認が必要 |
| Split 信頼性 | 長時間 clock drift、再接続直後の入力、実 BLE jitter、机上距離の復帰、+8 dBm の距離・電流比較 |
| バッテリー | 完全放電、DMM 比較、動作/idle/System OFF 電流、実使用時間の測定 |
| 状態 LED | 放電機で赤い低電圧表示を確認し、純正相当の充電/満充電表示を実装 |
| 純正 2.4 GHz/ドングル | USB 受信機、外部 nRF24L01、ESB、ドングル pairing、別 numpad 通信は未実装 |
| NocFree Link 追加機能 | バッテリー表示は unavailable。Quick Text の保存・削除・実行は未実装 |
| その他のツール | 純正 updater と ZMK Studio は未対応で、現在の必須範囲外 |
| 対応 platform | Bluetooth host は Windows 11 と Android のみ確認。macOS、iOS、Linux、第3 host は未検証 |

純正比較とバッテリー校正手順は [ROADMAP_ja.md](ROADMAP_ja.md)、詳細な実験記録は
[PROGRESS_ja.md](PROGRESS_ja.md) にまとめています。

## 実装済み機能

- **ANSI 入力:** 左37キー、右47キー、左右 Fn、media/system キー、韓国語
  Windows 特殊キー、暗号化 BLE split。
- **USB/Bluetooth HID:** 物理スイッチによる出力選択、BLE 自動再接続、CCCD
  状態保存、選択状態を保持する3つの pairing slot。
- **NocFree Link:** 8×84 keymap、実行可能な16 hotkey、CRC 付き flash 保存、
  削除、既定値復元。
- **復旧:** 左右独立 1200-baud CDC DFU、長押し Fn DFU、UF2 起動、
  Rust ↔ 純正 V2.3.0 の復元経路。
- **バックライト:** 左右 on/off と 0/20/40/60/80/100% 同期、10 kHz PWM、
  見やすい既定 linear 段階、再現可能な perceptual A/B 曲線、30秒 idle 消灯、
  最初のキーで復帰。`Fn+F5` が暗く、`Fn+F6` が明るくなります。
- **電源動作:** interrupt ベース idle scan と 250 ms safety scan、測定時だけ
  battery divider を有効化、バッテリー時は左が5分で System OFF。
- **バッテリー表示:** 純正 V2.3.0 の換算・filter と `Fn+I` 出力。満充電の左右で
  `L 100 R 100` を確認。
- **状態・診断:** 継続する青 pairing 表示、自動検査済みの赤低電圧ロジック、
  両 USB から読める直近32件の split event。
- **安全な flash 範囲:** SoftDevice、永続保存、純正 filesystem、UF2 bootloader
  領域を保護。

以前の ANSI は84キー、Wired/Bluetooth、Windows 11・Android の multi-pairing、
物理スイッチ、NocFree Link、バックライト同期・消灯、電源 wake、左右 DFU、純正
復元を実機で確認しました。この結果は現在の `develop` 成果物の実機検証を
代替しません。最新状態は [HANDOFF_ja.md](HANDOFF_ja.md) を参照してください。

## 既定のショートカット

左右 Fn は同じ layer を使います。以下は NocFree Link で変更する前の既定値です。

| キー | 操作 | 動作 |
|---|---|---|
| `Fn+Esc` | 即時 | 左 application を再起動。DFU ではない |
| `Fn+F1` / `Fn+F2` | 即時 | 画面輝度を下げる / 上げる |
| `Fn+F3` / `Fn+F4` | 即時 | Mission Control/Task View、Spotlight/Search |
| `Fn+F5` / `Fn+F6` | 即時 | 左右のキーボード照明を20%下げる / 上げる |
| `Fn+F7` / `Fn+F8` / `Fn+F9` | 即時 | 前曲 / 再生・一時停止 / 次曲 |
| `Fn+F10` / `Fn+F11` / `Fn+F12` | 即時 | mute / 音量を下げる / 上げる |
| `Fn+1` / `Fn+2` / `Fn+3` | 短押し | Bluetooth pairing slot 1 / 2 / 3 を選択 |
| `Fn+1` / `Fn+2` / `Fn+3` | 1秒 | 対象 bond を削除し新規 pairing を開始 |
| `Fn+5` | 3秒 | **左** UF2 bootloader へ入る |
| `Fn+0` | 3秒 | **右** UF2 bootloader へ入る |
| `Fn+Tab` | 即時 | 左右バックライトを切り替える |
| `Fn+I` | 3秒 | 有効出力へ `L {left %} R {right %}` を入力 |
| `Fn+Delete` | 3秒 | 右 DFU 互換キー。新しい手順は `Fn+0` を推奨 |
| `Fn+M` | 短押し / 1秒 | M 入力 / macOS モード保存 |
| `Fn+N` | 短押し / 1秒 | N 入力 / Windows モード保存 |

表にない Fn 組み合わせは元のキーを入力します。`Fn+6` と `Fn+7` は profile
削除や出力切替を行いません。長押し `Fn+1/2/3` と物理スイッチを使用します。
`Fn+U` と `Fn+B` も元の文字を入力します。

## 技術資料

### 左右入力順序

右 snapshot は原時刻・sequence・reconciliation 情報を送り、左は split-ready
前に clock offset を計算して60秒ごとに更新し、local/remote event を1つの
queue で統合します。

順序待ち時間は実機検証済みの **3 ms** から **8 ms** 候補を経て、現在の
**5 ms** へ変更しました。`src/scanner.rs` の `REORDER_WINDOW_MS` で調整できます。
小さい値は遅延を減らし、大きい値は BLE transport jitter の余裕を増やします。
変更後は左右を同時に再ビルド・フラッシュし、USB と Bluetooth の高速な
L-R-L/R-L-R 入力を確認してください。compile 成功だけでは安全値を決められません。

### Bluetooth profile と split 接続

pairing slot 1/2/3 は1つの BLE identity 配下の bond であり、3台の別キーボード
ではありません。`NocFree 1/2/3` は選択 slot の advertising 名です。現在の
host と bond 済みの slot を選び、切替のたびに Windows device を削除しません。

右 split は 1M BLE、暗号化、7.5 ms 接続間隔、latency 30、4秒 supervision
timeout、ATT MTU 23、段階式 advertising、+8 dBm TX を使います。復帰は改善
しましたが、距離・電力・長期動作は上記の未検証項目に残っています。

### 電源、診断、復旧

- NocFree キー入力が30秒ないと、USB 接続中でも左右バックライトが消灯します。
- バッテリー時は左が5分で System OFF になります。USB と pairing 中は入りません。
  USB を接続するか、再接続まで左キーを長押しして復帰させます。
- 右は BLE split 自動再接続のため低活動 System ON を維持します。右 System OFF
  は未実装です。
- 状態を変えずに split log を読めます。

```powershell
& '.\tools\read-split-diagnostics.ps1' -Role Left
& '.\tools\read-split-diagnostics.ps1' -Role Right
```

診断は 115200 baud、DFU touch は 1200 baud です。未確認 PCB pad を短絡せず、
[RECOVERY_ja.md](RECOVERY_ja.md) に従ってください。

## 関連文書

- [復旧と純正ファームウェアへの復元](RECOVERY_ja.md)
- [配列別ガイド](LAYOUTS_ja.md)
- [未実装機能とバッテリー校正計画](ROADMAP_ja.md)
- [詳細な開発・検証記録](PROGRESS_ja.md)
- [継続作業の引き継ぎ](HANDOFF_ja.md)

ローカル `vendor/nrf-softdevice` と `vendor/nrf-softdevice-macro` は upstream
commit `b0ac850c0a5a05b8a5aef4f752b48115755b8542` ベースです。それぞれの
`README.nocfree_ja.md` に 1M PHY と secure CCCD 変更理由があります。
