# NocFree local patch

[English](README.nocfree.md) · [한국어](README.nocfree_ko.md) · [日本語](README.nocfree_ja.md)

この directory は `embassy-rs/nrf-softdevice` commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542` からコピーしました。

upstream の characteristic `security = "..."` option は value attribute には
security を適用しますが、自動生成される CCCD/SCCD には適用しません。local patch は
同じ security mode を descriptor metadata にも適用します。NocFree は encrypted
split-state notification subscription の保護にこの動作を使用します。
