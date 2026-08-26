# NocFree local patch

[English](README.nocfree.md) · [한국어](README.nocfree_ko.md) · [日本語](README.nocfree_ja.md)

This directory is copied from `embassy-rs/nrf-softdevice` commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`.

Upstream's `security = "..."` characteristic option applies security to the
value attribute but not to the automatically-created CCCD/SCCD. The local
patch applies the same security mode to that descriptor metadata. NocFree uses
this for the encrypted split-state notification subscription.
