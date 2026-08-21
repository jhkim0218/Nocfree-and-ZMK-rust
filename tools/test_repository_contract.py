import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


class RepositoryContractTests(unittest.TestCase):
    def test_linker_and_storage_boundaries_preserve_factory_regions(self) -> None:
        memory = read("memory.x")
        records = read("src/bond_record.rs")
        self.assertIn("ORIGIN = 0x00027000, LENGTH = 0x0003E000", memory)
        self.assertIn("pub const STORAGE_START: u32 = 0x65000", records)
        self.assertIn("pub const STORAGE_END: u32 = 0x6d000", records)

    def test_dfu_uses_softdevice_system_calls(self) -> None:
        platform = read("src/platform.rs")
        cargo = read("Cargo.toml")
        self.assertIn("sd_power_gpregret_clr(0, 0xff)", platform)
        self.assertIn("sd_power_gpregret_set(0, 0x57)", platform)
        self.assertIn("#[panic_handler]", platform)
        self.assertNotIn("panic-halt", cargo)
        self.assertNotIn("pac::POWER", platform)

    def test_all_ble_links_are_pinned_to_1m(self) -> None:
        gap = read("vendor/nrf-softdevice/src/ble/gap.rs")
        self.assertIn("rx_phys: raw::BLE_GAP_PHY_1MBPS as u8", gap)
        self.assertIn("tx_phys: raw::BLE_GAP_PHY_1MBPS as u8", gap)

    def test_split_state_and_dfu_command_require_encryption(self) -> None:
        split = read("src/split_ble.rs")
        macro = read("vendor/nrf-softdevice-macro/src/lib.rs")
        self.assertEqual(split.count('security = "justworks"'), 2)
        self.assertIn("Metadata::new(props)", macro)
        self.assertIn(".security(#security_inner)", macro)

    def test_split_central_has_an_smp_security_slot(self) -> None:
        platform = read("src/platform.rs")
        self.assertIn("central_sec_count: 1", platform)

    def test_only_left_exposes_host_hid(self) -> None:
        central = read("src/bin/central.rs")
        right = read("src/bin/right.rs")
        self.assertIn("HidWriter", central)
        self.assertNotIn("HidWriter", right)
        self.assertIn("CdcAcmClass", central)
        self.assertIn("CdcAcmClass", right)

    def test_left_exposes_nocfree_link_and_uses_its_persisted_keymap(self) -> None:
        central = read("src/bin/central.rs")
        cargo_config = read(".cargo/config.toml")
        storage = read("src/bond_store.rs")
        protocol = read("src/link_protocol.rs")
        self.assertIn("Config::new(0x2886, 0x8029)", central)
        self.assertIn('usb_config.product = Some("NocFree & ANSI")', central)
        self.assertIn("LinkUsbClass::new", central)
        self.assertIn("BONDS.key_action(layer, raw)", central)
        self.assertIn('EMBASSY_USB_MAX_INTERFACE_COUNT = "5"', cargo_config)
        self.assertIn('EMBASSY_USB_MAX_HANDLER_COUNT = "5"', cargo_config)
        self.assertIn("LINK_KEYMAP_PAGE", storage)
        self.assertIn("const SET_HOTKEY: u8 = 52", protocol)
        self.assertIn("const GET_TEXT: u8 = 49", protocol)

    def test_cross_half_updates_share_one_fifo(self) -> None:
        central = read("src/bin/central.rs")
        self.assertIn("static INPUT_STATE: KeyState<32>", central)
        self.assertIn("INPUT_STATE.wait_changed().await", central)
        self.assertNotIn("LOCAL_STATE.wait_changed()", central)
        self.assertNotIn("REMOTE_STATE.wait_changed()", central)

    def test_split_connection_uses_zmk_timing(self) -> None:
        central = read("src/bin/central.rs")
        protocol = read("src/split_protocol.rs")
        self.assertIn("pub const CONNECTION_INTERVAL_UNITS: u16 = 6", protocol)
        self.assertIn("pub const CONNECTION_LATENCY: u16 = 30", protocol)
        self.assertIn("pub const CONNECTION_TIMEOUT_UNITS: u16 = 400", protocol)
        self.assertIn(
            "connect_config.conn_params.min_conn_interval = CONNECTION_INTERVAL_UNITS",
            central,
        )
        self.assertIn(
            "connect_config.conn_params.max_conn_interval = CONNECTION_INTERVAL_UNITS",
            central,
        )

    def test_firmware_only_configures_published_i2c_pins(self) -> None:
        central = read("src/bin/central.rs")
        right = read("src/bin/right.rs")
        for firmware in (central, right):
            self.assertIn("peripherals.P0_11", firmware)
            self.assertIn("peripherals.P1_09", firmware)
            self.assertNotIn("Output::new", firmware)

    def test_scanner_retries_transient_expander_startup_failures(self) -> None:
        scanner = read("src/hardware_scanner.rs")
        self.assertIn("while expanders.configure_and_verify().await.is_err()", scanner)
        self.assertNotIn("core::future::pending", scanner)

    def test_both_halves_recover_i2c_before_starting_twim(self) -> None:
        scanner = read("src/hardware_scanner.rs")
        self.assertIn("for _ in 0..9", scanner)
        self.assertIn("sda.set_high()", scanner)
        for path in ("src/bin/central.rs", "src/bin/right.rs"):
            firmware = read(path)
            recovery = firmware.index("hardware_scanner::recover_i2c_bus(")
            twim = firmware.index("let twim = Twim::new(")
            self.assertLess(recovery, twim)


if __name__ == "__main__":
    unittest.main()
