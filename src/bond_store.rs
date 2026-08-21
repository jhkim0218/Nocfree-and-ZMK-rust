use core::cell::RefCell;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};
use nrf_softdevice::Flash;
use nrf_softdevice::ble::security::SecurityHandler;
use nrf_softdevice::ble::{
    Connection, EncryptionInfo, IdentityKey, MasterId, SecurityMode, gatt_server,
};
use nrf_softdevice::raw;

use crate::bond_record::{
    BondRecord, PAGE_SIZE, PROFILE_COUNT, RECORD_BYTES, SETTINGS_PAGE, SPLIT_BOND_SLOT, SPLIT_PAGE,
    STORAGE_START, SYS_ATTR_CAPACITY, decode_bond, decode_selected_profile, encode_bond,
    encode_selected_profile,
};
use crate::keymap::Action;
use crate::link_keymap::{LINK_KEYMAP_PAGE, LINK_KEYMAP_RECORD_BYTES, LinkKeymap};
use crate::link_protocol::{LinkResponse, handle_request};

#[derive(Clone, Copy)]
struct Peer {
    master_id: MasterId,
    key: EncryptionInfo,
    peer_id: IdentityKey,
    sys_attr_len: u8,
    sys_attrs: [u8; SYS_ATTR_CAPACITY],
}

#[repr(C, align(4))]
struct AlignedRecord([u8; RECORD_BYTES]);

#[repr(C, align(4))]
struct AlignedKeymap([u8; LINK_KEYMAP_RECORD_BYTES]);

pub struct BondStore {
    peers: Mutex<CriticalSectionRawMutex, RefCell<[Option<Peer>; PROFILE_COUNT]>>,
    split_peer: Mutex<CriticalSectionRawMutex, RefCell<Option<Peer>>>,
    keymap: Mutex<CriticalSectionRawMutex, RefCell<LinkKeymap>>,
    selected: AtomicU8,
    dirty_pages: AtomicU8,
    save_request: Signal<CriticalSectionRawMutex, ()>,
    ready: AtomicBool,
}

impl Default for BondStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BondStore {
    pub const fn new() -> Self {
        Self {
            peers: Mutex::new(RefCell::new([None; PROFILE_COUNT])),
            split_peer: Mutex::new(RefCell::new(None)),
            keymap: Mutex::new(RefCell::new(LinkKeymap::new())),
            selected: AtomicU8::new(0),
            dirty_pages: AtomicU8::new(0),
            save_request: Signal::new(),
            ready: AtomicBool::new(false),
        }
    }

    pub async fn wait_ready(&self) {
        while !self.ready.load(Ordering::Acquire) {
            Timer::after(Duration::from_millis(1)).await;
        }
    }

    pub fn selected(&self) -> u8 {
        self.selected.load(Ordering::Acquire).min(4)
    }

    pub fn select(&self, profile: u8) {
        let profile = profile.min(4);
        self.selected.store(profile, Ordering::Release);
        self.request_save(SETTINGS_PAGE);
    }

    pub fn clear_selected(&self) {
        let profile = self.selected();
        self.peers
            .lock(|peers| peers.borrow_mut()[profile as usize] = None);
        self.request_save(profile);
    }

    pub fn has_split_peer(&self) -> bool {
        self.split_peer().is_some()
    }

    pub fn clear_split_peer(&self) {
        self.set_split_peer(None);
        self.request_save(SPLIT_PAGE);
    }

    pub fn key_action(&self, layer: u8, raw: usize) -> Action {
        self.keymap
            .lock(|keymap| keymap.borrow().action(layer as usize, raw))
    }

    pub fn handle_link_frame(&self, frame: &[u8]) -> Option<LinkResponse> {
        let response = self
            .keymap
            .lock(|keymap| handle_request(frame, &mut keymap.borrow_mut()))?;
        if response.changed {
            self.request_save(LINK_KEYMAP_PAGE);
        }
        Some(response)
    }

    fn selected_peer(&self) -> Option<Peer> {
        let profile = self.selected() as usize;
        self.peers.lock(|peers| peers.borrow()[profile])
    }

    fn set_peer(&self, profile: u8, peer: Option<Peer>) {
        self.peers
            .lock(|peers| peers.borrow_mut()[profile as usize] = peer);
    }

    fn split_peer(&self) -> Option<Peer> {
        self.split_peer.lock(|peer| *peer.borrow())
    }

    fn set_split_peer(&self, peer: Option<Peer>) {
        self.split_peer.lock(|slot| *slot.borrow_mut() = peer);
    }

    fn request_save(&self, page: u8) {
        self.dirty_pages.fetch_or(1 << page, Ordering::AcqRel);
        self.save_request.signal(());
    }
}

pub struct SplitSecurity {
    store: &'static BondStore,
}

impl SplitSecurity {
    pub const fn new(store: &'static BondStore) -> Self {
        Self { store }
    }
}

impl SecurityHandler for BondStore {
    fn can_bond(&self, connection: &Connection) -> bool {
        match self.selected_peer() {
            Some(peer) => peer.peer_id.is_match(connection.peer_address()),
            None => true,
        }
    }

    fn on_security_update(&self, connection: &Connection, security_mode: SecurityMode) {
        if matches!(security_mode, SecurityMode::NoAccess | SecurityMode::Open) {
            return;
        }
        let occupied_by_other_peer = self
            .selected_peer()
            .is_some_and(|peer| !peer.peer_id.is_match(connection.peer_address()));
        if occupied_by_other_peer {
            let connection = connection.clone();
            let _ = connection.disconnect();
        }
    }

    fn on_bonded(
        &self,
        _connection: &Connection,
        master_id: MasterId,
        key: EncryptionInfo,
        peer_id: IdentityKey,
    ) {
        let profile = self.selected();
        self.set_peer(
            profile,
            Some(Peer {
                master_id,
                key,
                peer_id,
                sys_attr_len: 0,
                sys_attrs: [0; SYS_ATTR_CAPACITY],
            }),
        );
        self.request_save(profile);
    }

    fn get_key(&self, _connection: &Connection, master_id: MasterId) -> Option<EncryptionInfo> {
        self.selected_peer()
            .and_then(|peer| (peer.master_id == master_id).then_some(peer.key))
    }

    fn get_peripheral_key(&self, connection: &Connection) -> Option<(MasterId, EncryptionInfo)> {
        self.selected_peer().and_then(|peer| {
            peer.peer_id
                .is_match(connection.peer_address())
                .then_some((peer.master_id, peer.key))
        })
    }

    fn save_sys_attrs(&self, connection: &Connection) {
        let Some(mut peer) = self.selected_peer() else {
            return;
        };
        if !peer.peer_id.is_match(connection.peer_address()) {
            return;
        }

        let Ok(length) = gatt_server::get_sys_attrs(connection, &mut peer.sys_attrs) else {
            return;
        };
        peer.sys_attr_len = length.min(SYS_ATTR_CAPACITY) as u8;
        let profile = self.selected();
        self.set_peer(profile, Some(peer));
        self.request_save(profile);
    }

    fn load_sys_attrs(&self, connection: &Connection) {
        let peer = self.selected_peer();
        let attrs = peer.as_ref().and_then(|peer| {
            peer.peer_id
                .is_match(connection.peer_address())
                .then_some(&peer.sys_attrs[..peer.sys_attr_len as usize])
        });
        let _ = gatt_server::set_sys_attrs(connection, attrs);
    }
}

impl SecurityHandler for SplitSecurity {
    fn can_bond(&self, connection: &Connection) -> bool {
        match self.store.split_peer() {
            Some(peer) => peer.peer_id.is_match(connection.peer_address()),
            None => true,
        }
    }

    fn on_security_update(&self, connection: &Connection, security_mode: SecurityMode) {
        if matches!(security_mode, SecurityMode::NoAccess | SecurityMode::Open) {
            return;
        }
        let occupied_by_other_peer = self
            .store
            .split_peer()
            .is_some_and(|peer| !peer.peer_id.is_match(connection.peer_address()));
        if occupied_by_other_peer {
            let connection = connection.clone();
            let _ = connection.disconnect();
        }
    }

    fn on_bonded(
        &self,
        _connection: &Connection,
        master_id: MasterId,
        key: EncryptionInfo,
        peer_id: IdentityKey,
    ) {
        self.store.set_split_peer(Some(Peer {
            master_id,
            key,
            peer_id,
            sys_attr_len: 0,
            sys_attrs: [0; SYS_ATTR_CAPACITY],
        }));
        self.store.request_save(SPLIT_PAGE);
    }

    fn get_key(&self, _connection: &Connection, master_id: MasterId) -> Option<EncryptionInfo> {
        self.store
            .split_peer()
            .and_then(|peer| (peer.master_id == master_id).then_some(peer.key))
    }

    fn get_peripheral_key(&self, connection: &Connection) -> Option<(MasterId, EncryptionInfo)> {
        self.store.split_peer().and_then(|peer| {
            peer.peer_id
                .is_match(connection.peer_address())
                .then_some((peer.master_id, peer.key))
        })
    }

    fn save_sys_attrs(&self, connection: &Connection) {
        let Some(mut peer) = self.store.split_peer() else {
            return;
        };
        if !peer.peer_id.is_match(connection.peer_address()) {
            return;
        }

        let Ok(length) = gatt_server::get_sys_attrs(connection, &mut peer.sys_attrs) else {
            return;
        };
        peer.sys_attr_len = length.min(SYS_ATTR_CAPACITY) as u8;
        self.store.set_split_peer(Some(peer));
        self.store.request_save(SPLIT_PAGE);
    }

    fn load_sys_attrs(&self, connection: &Connection) {
        let peer = self.store.split_peer();
        let attrs = peer.as_ref().and_then(|peer| {
            peer.peer_id
                .is_match(connection.peer_address())
                .then_some(&peer.sys_attrs[..peer.sys_attr_len as usize])
        });
        let _ = gatt_server::set_sys_attrs(connection, attrs);
    }
}

pub async fn run_storage(mut flash: Flash, store: &'static BondStore) -> ! {
    let mut buffer = AlignedRecord([0; RECORD_BYTES]);
    let mut keymap_buffer = AlignedKeymap([0; LINK_KEYMAP_RECORD_BYTES]);
    for profile in 0..PROFILE_COUNT as u8 {
        if flash
            .read(page_address(profile), &mut buffer.0)
            .await
            .is_ok()
        {
            store.set_peer(
                profile,
                decode_bond(profile, &buffer.0).map(peer_from_record),
            );
        }
    }
    if flash
        .read(page_address(SETTINGS_PAGE), &mut buffer.0)
        .await
        .is_ok()
    {
        store.selected.store(
            decode_selected_profile(&buffer.0).unwrap_or(0),
            Ordering::Release,
        );
    }
    if flash
        .read(page_address(SPLIT_PAGE), &mut buffer.0)
        .await
        .is_ok()
    {
        store.set_split_peer(decode_bond(SPLIT_BOND_SLOT, &buffer.0).map(peer_from_record));
    }
    if flash
        .read(page_address(LINK_KEYMAP_PAGE), &mut keymap_buffer.0)
        .await
        .is_ok()
        && let Some(keymap) = LinkKeymap::decode(&keymap_buffer.0)
    {
        store.keymap.lock(|current| *current.borrow_mut() = keymap);
    }
    store.ready.store(true, Ordering::Release);

    loop {
        store.save_request.wait().await;
        let dirty = store.dirty_pages.swap(0, Ordering::AcqRel);
        for page in 0..=LINK_KEYMAP_PAGE {
            if dirty & (1 << page) == 0 {
                continue;
            }
            let address = page_address(page);
            if flash.erase(address, address + PAGE_SIZE).await.is_err() {
                store.request_save(page);
                Timer::after(Duration::from_millis(100)).await;
                continue;
            }

            if page == LINK_KEYMAP_PAGE {
                keymap_buffer.0 = store.keymap.lock(|keymap| keymap.borrow().encode());
                if flash.write(address, &keymap_buffer.0).await.is_err() {
                    store.request_save(page);
                    Timer::after(Duration::from_millis(100)).await;
                }
                continue;
            }

            let encoded = if page == SETTINGS_PAGE {
                Some(encode_selected_profile(store.selected()))
            } else if page == SPLIT_PAGE {
                store
                    .split_peer
                    .lock(|peer| *peer.borrow())
                    .map(|peer| encode_bond(SPLIT_BOND_SLOT, &record_from_peer(peer)))
            } else {
                store
                    .peers
                    .lock(|peers| peers.borrow()[page as usize])
                    .map(|peer| encode_bond(page, &record_from_peer(peer)))
            };
            if let Some(encoded) = encoded {
                buffer.0 = encoded;
                if flash.write(address, &buffer.0).await.is_err() {
                    store.request_save(page);
                    Timer::after(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

const fn page_address(page: u8) -> u32 {
    STORAGE_START + page as u32 * PAGE_SIZE
}

fn record_from_peer(peer: Peer) -> BondRecord {
    let identity = peer.peer_id.as_raw();
    BondRecord {
        master_ediv: peer.master_id.ediv,
        master_rand: peer.master_id.rand,
        ltk: peer.key.ltk,
        encryption_flags: peer.key.flags,
        irk: identity.id_info.irk,
        address_flags: peer.peer_id.addr.flags,
        address: peer.peer_id.addr.bytes,
        sys_attr_len: peer.sys_attr_len,
        sys_attrs: peer.sys_attrs,
    }
}

fn peer_from_record(record: BondRecord) -> Peer {
    Peer {
        master_id: MasterId {
            ediv: record.master_ediv,
            rand: record.master_rand,
        },
        key: EncryptionInfo {
            ltk: record.ltk,
            flags: record.encryption_flags,
        },
        peer_id: IdentityKey::from_raw(raw::ble_gap_id_key_t {
            id_info: raw::ble_gap_irk_t { irk: record.irk },
            id_addr_info: raw::ble_gap_addr_t {
                _bitfield_1: raw::ble_gap_addr_t::new_bitfield_1(
                    record.address_flags & 1,
                    record.address_flags >> 1,
                ),
                addr: record.address,
            },
        }),
        sys_attr_len: record.sys_attr_len,
        sys_attrs: record.sys_attrs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clearing_split_peer_marks_its_flash_page_for_erasure() {
        let store = BondStore::new();

        store.clear_split_peer();

        assert!(!store.has_split_peer());
        assert_eq!(store.dirty_pages.load(Ordering::Acquire), 1 << SPLIT_PAGE);
    }
}
