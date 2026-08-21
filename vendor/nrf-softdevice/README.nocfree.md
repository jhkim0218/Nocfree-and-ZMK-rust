# NocFree local patch

This directory is copied from `embassy-rs/nrf-softdevice` commit
`b0ac850c0a5a05b8a5aef4f752b48115755b8542`.

The behavioral change in this crate is in `src/ble/gap.rs`: PHY update requests
are answered with 1 Mbps for both RX and TX. This preserves the original
NocFree firmware's 1M-only BLE contract even when a peer requests 2M.

The package's sibling path dependencies were changed to the same pinned Git
revision so this single crate can remain vendored without copying unrelated
SoftDevice bindings.

The sibling `nrf-softdevice-macro` crate is also local so secured notification
characteristics protect their CCCDs; its own patch note has the details.
