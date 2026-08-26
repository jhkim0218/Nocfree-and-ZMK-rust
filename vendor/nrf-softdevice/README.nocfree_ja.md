# NocFree local patch

[English](README.nocfree.md) · [한국어](README.nocfree_ko.md) · [日本語](README.nocfree_ja.md)

この directory は `embassy-rs/nrf-softdevice` commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542` からコピーしました。

動作変更は `src/ble/gap.rs` にあります。peer が 2M を要求しても PHY update に
RX/TX とも 1 Mbps で応答し、元の NocFree firmware の 1M-only BLE contract を
維持します。

sibling path dependency は同じ固定 Git revision に変更しました。そのため、
無関係な SoftDevice binding をすべてコピーせず、この crate だけを vendor として
保持できます。

secured notification characteristic の CCCD も保護するため、sibling の
`nrf-softdevice-macro` crate も local に置いています。詳細はそちらの patch noteを
参照してください。
