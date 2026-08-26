# 物理配列バリアント

[English](LAYOUTS.md) · [한국어](LAYOUTS_ko.md) · [日本語](LAYOUTS_ja.md)

ファームウェアはコンパイル時に ANSI、ISO、JIS、KR のいずれか1つだけを
選択します。ANSI が既定で、メンテナの実機で検証された唯一の配列です。

| 配列 | ソース | 左 | 右 | 根拠 | 状態 |
|---|---|---:|---:|---|---|
| ANSI | `src/keymap/ansi.rs` | 37 | 47 | 元の community ZMK とローカル実機 | 安定候補・実機履歴あり |
| ISO | `src/keymap/iso.rs` | 38 | 47 | 公式 updater ISO image | Experimental・実機未検証 |
| JIS | `src/keymap/jis.rs` | 37 | 48 | `electricdoc187` の `jis-custom` scan map | Experimental・Rust 実機未検証 |
| KR | `src/keymap/kr.rs` | 39 | 50 | 公式 updater/product と `0x21/P0` | Experimental・実機未検証 |

共通動作は `src/keymap.rs` にあり、物理キー数、座標変換、HID usage、Fn 位置、
PCA9555 address だけを各配列 module が所有します。保存される NocFree Link
keymap は version、layout ID、key count、CRC を持つため、別配列の記録を誤って
適用しません。layout ID を持たない version 3 は version 4 の初回起動で既定へ
戻ります。BLE host bond と split bond は別記録なので消去されません。

## ビルド

Windows、macOS、Linux で次のいずれかを実行します。

```text
python3 -B tools/build_release.py --layout ANSI
python3 -B tools/build_release.py --layout ISO
python3 -B tools/build_release.py --layout JIS
python3 -B tools/build_release.py --layout KR
```

環境によって `python3` を `python` に置き換えてください。Windows の
`tools/build-release.ps1 -Layout ANSI` は同じ Python builder を呼ぶ wrapper です。
ANSI は `firmware`、他は `firmware/experimental` に出力されます。同じ配列、
同じ build の Left/Right だけを一組としてフラッシュしてください。

## 制限

- ISO/JIS/KR は compile、synthetic input、artifact test が通っても実機検証ではありません。
- JIS の usage 0x87/0x89/0x8a と右48番目のキーに合わせ、HID report と
  debouncer 容量を拡張済みですが、Rust 実機試験は未実施です。
- 参考 ZMK の左 Eisu は tap Muhenkan / hold Fn です。初期 Rust JIS は tester が
  いないため Fn のみとし、tap/hold は保留しています。
- host の keyboard layout/IME は OS の設定で、firmware は自動変更しません。
- 異なる物理配列へ Experimental image を書き込んではいけません。

## 次の実機試験

まず ANSI の matching pair を右、左の順で更新し、84キー、両 Fn、Wired USB、
Windows 11 BLE reconnect、backlight 同期と新しい明るさ曲線、`Fn+5`/`Fn+0`
復旧、NocFree Link の変更と再起動後保存を確認します。その回帰が完了してから、
各実機を所有する協力者が ISO、JIS、KR を試します。

参照: <https://github.com/NocFreeKB/NocFree-and-zmk>、
<https://github.com/electricdoc187/NocFree-and-zmk/tree/jis-custom>、
<https://www.nocfree.com/products/nocfree-and-reservation>。
