// SPAGHETTI ALERT. This should never panic but HOLY SHIT IS IT UGLY. Scroll down if you dare.
// 11/28/2022 update: I hate myself

use std::mem;
use sysinfo::{PidExt, ProcessExt, System, SystemExt};
use windows::Win32::{
    Foundation::{self, HANDLE, HINSTANCE},
    System::{Diagnostics::Debug, Memory, ProcessStatus, Threading},
};

const VERSION_DATA: [u8; 64usize] = [
    0x4cu8, 0x89u8, 0x6fu8, 0x10u8, 0x48u8, 0xc7u8, 0x47u8, 0x18u8, 0x07u8, 0x00u8, 0x00u8, 0x00u8,
    0x66u8, 0x44u8, 0x89u8, 0x2fu8, 0x48u8, 0x8bu8, 0x54u8, 0x24u8, 0x60u8, 0x48u8, 0x83u8, 0xfau8,
    0x08u8, 0x72u8, 0x37u8, 0x48u8, 0x8du8, 0x14u8, 0x55u8, 0x02u8, 0x00u8, 0x00u8, 0x00u8, 0x48u8,
    0x8bu8, 0x4cu8, 0x24u8, 0x48u8, 0x48u8, 0x8bu8, 0xc1u8, 0x48u8, 0x81u8, 0xfau8, 0x00u8, 0x10u8,
    0x00u8, 0x00u8, 0x72u8, 0x19u8, 0x48u8, 0x83u8, 0xc2u8, 0x27u8, 0x48u8, 0x8bu8, 0x49u8, 0xf8u8,
    0x48u8, 0x2bu8, 0xc1u8, 0x48u8,
];

// I have to use the same number of bytes for each opcode because of spaghetti. I hate this
pub const NO_SEARCH_LOCKING_DATA: [(usize, [u8; 18usize], [u8; 18usize]); 3usize] = [
    (
        0x3e7c86usize,
        [
            0x0fu8, 0xb6u8, 0x8bu8, 0x41u8, 0x66u8, 0x02u8, 0x00u8, 0x8bu8, 0xc7u8, 0x83u8, 0xf9u8,
            0x01u8, 0x0fu8, 0x1fu8, 0x00u8, 0xbau8, 0x04u8, 0x00u8,
        ],
        [
            0x3bu8, 0x8bu8, 0xd0u8, 0x05u8, 0x00u8, 0x00u8, 0x8bu8, 0xc7u8, 0x0fu8, 0xb6u8, 0x8bu8,
            0x41u8, 0x66u8, 0x02u8, 0x00u8, 0xbau8, 0x04u8, 0x00u8,
        ],
    ),
    (
        0x3eb2aausize,
        [
            0x0fu8, 0xb6u8, 0xfau8, 0x48u8, 0x8bu8, 0xd9u8, 0x80u8, 0xb9u8, 0x41u8, 0x66u8, 0x02u8,
            0x00u8, 0x01u8, 0x0fu8, 0x1fu8, 0x44u8, 0x00u8, 0x00u8,
        ],
        [
            0x8bu8, 0x81u8, 0xd4u8, 0x05u8, 0x00u8, 0x00u8, 0x0fu8, 0xb6u8, 0xfau8, 0x48u8, 0x8bu8,
            0xd9u8, 0x3bu8, 0x81u8, 0xd0u8, 0x05u8, 0x00u8, 0x00u8,
        ],
    ),
    (
        0x3eb4d2usize,
        [
            0x80u8, 0xb9u8, 0x41u8, 0x66u8, 0x02u8, 0x00u8, 0x01u8, 0x0fu8, 0x1fu8, 0x44u8, 0x00u8,
            0x00u8, 0x0fu8, 0x85u8, 0x82u8, 0x00u8, 0x00u8, 0x00u8,
        ],
        [
            0x8bu8, 0x81u8, 0xd4u8, 0x05u8, 0x00u8, 0x00u8, 0x3bu8, 0x81u8, 0xd0u8, 0x05u8, 0x00u8,
            0x00u8, 0x0fu8, 0x85u8, 0x82u8, 0x00u8, 0x00u8, 0x00u8,
        ],
    ),
];

pub const CHTHONIA_FILTER_DATA: (usize, [u8; 7usize], [u8; 7usize]) = (
    0x3ea759usize,
    [0xb8u8, 0x07u8, 0x00u8, 0x00u8, 0x00u8, 0x66u8, 0x90u8],
    [0x41u8, 0x8bu8, 0x41u8, 0x08u8, 0x4du8, 0x8bu8, 0xd1u8],
);

pub const ACCURATE_TEMP_FILTER_DATA: (usize, [u8; 1usize], [u8; 1usize]) =
    (0x3ea85busize, [0x48u8], [0x4cu8]);

type Opcodes = Vec<(usize, Vec<u8>, Vec<u8>)>;

#[derive(Debug, Default)]
pub struct CompactPatch(Opcodes);

impl CompactPatch {
    pub fn new(data: Opcodes) -> Self {
        CompactPatch(data)
    }

    pub fn enable(&self, handler: &Handler) {
        for opcode in self.0.clone() {
            let address = opcode.0;
            let bytes = opcode.1;

            unsafe {
                let mut old_protection = Memory::PAGE_PROTECTION_FLAGS(0u32);

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + address) as _,
                    6usize,
                    Memory::PAGE_EXECUTE_READWRITE,
                    &mut old_protection,
                );

                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + address) as _,
                    bytes.as_ptr() as _,
                    bytes.len(),
                    None,
                );

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + address) as _,
                    6usize,
                    old_protection,
                    &mut old_protection,
                );
            };
        }
    }

    pub fn disable(&self, handler: &Handler) {
        for opcode in self.0.clone() {
            let address = opcode.0;
            let bytes = opcode.2;

            unsafe {
                let mut old_protection = Memory::PAGE_PROTECTION_FLAGS(0u32);

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + address) as _,
                    6usize,
                    Memory::PAGE_EXECUTE_READWRITE,
                    &mut old_protection,
                );

                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + address) as _,
                    bytes.as_ptr() as _,
                    bytes.len(),
                    None,
                );

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + address) as _,
                    6usize,
                    old_protection,
                    &mut old_protection,
                );
            };
        }
    }
}

pub struct NoMaxSearchRadius();

impl NoMaxSearchRadius {
    const NMSR_DATA_OLD: (usize, u8, u8) = (0x3eb31eusize, 0xebu8, 0x74u8);
    const NMSR_DATA_NEW: [(usize, [u8; 8usize], [u8; 8usize]); 2usize] = [
        (
            0x3eb328usize,
            [
                0x66u8, 0x0fu8, 0x2fu8, 0x05u8, 0xd8u8, 0x51u8, 0xa1u8, 0xffu8,
            ],
            [
                0x66u8, 0x0fu8, 0x2fu8, 0x05u8, 0xf0u8, 0x01u8, 0x2bu8, 0x00u8,
            ],
        ),
        (
            0x3eb334usize,
            // We don't need this
            [0x00u8; 8usize],
            [
                0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x00u8, 0x59u8, 0x40u8,
            ],
        ),
    ];

    pub fn enable(&self, settings: (bool, bool, f32), handler: &Handler) {
        match settings.0 {
            true => unsafe {
                self.disable((false, false, 0.0f32), handler);

                let mut old_protection = Memory::PAGE_PROTECTION_FLAGS(0u32);
                let search_radius = match settings.1 {
                    true => f64::from(settings.2),
                    false => f64::from(settings.2 / 3.26156f32),
                }
                .to_le_bytes();

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    1usize,
                    Memory::PAGE_EXECUTE_READWRITE,
                    &mut old_protection,
                );

                // Temporarily store search radius in ESI Filter, since it's easier currently (I'm not gonna allocate an entire page just for 8 bytes)
                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address - 0x200000usize + 0x508usize) as _,
                    search_radius.as_ptr() as _,
                    mem::size_of_val(&search_radius),
                    None,
                );

                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_NEW[0usize].0) as _,
                    Self::NMSR_DATA_NEW[0usize].1.as_ptr() as _,
                    mem::size_of_val(&Self::NMSR_DATA_NEW[0usize].1),
                    None,
                );

                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_NEW[1usize].0) as _,
                    search_radius.as_ptr() as _,
                    mem::size_of_val(&search_radius),
                    None,
                );

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    1usize,
                    old_protection,
                    &mut old_protection,
                );
            },
            false => unsafe {
                self.disable((true, false, 0.0f32), handler);

                let mut old_protection = Memory::PAGE_PROTECTION_FLAGS(0u32);

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    1usize,
                    Memory::PAGE_EXECUTE_READWRITE,
                    &mut old_protection,
                );

                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    [Self::NMSR_DATA_OLD.1].as_ptr() as _,
                    mem::size_of_val(&Self::NMSR_DATA_OLD.1),
                    None,
                );

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    1usize,
                    old_protection,
                    &mut old_protection,
                );
            },
        };
    }

    pub fn disable(&self, settings: (bool, bool, f32), handler: &Handler) {
        match settings.0 {
            true => unsafe {
                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_NEW[0usize].0) as _,
                    Self::NMSR_DATA_NEW[0usize].2.as_ptr() as _,
                    mem::size_of_val(&Self::NMSR_DATA_NEW[0usize].2),
                    None,
                );

                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_NEW[1usize].0) as _,
                    Self::NMSR_DATA_NEW[1usize].2.as_ptr() as _,
                    mem::size_of_val(&Self::NMSR_DATA_NEW[1usize].2),
                    None,
                );
            },
            false => unsafe {
                let mut old_protection = Memory::PAGE_PROTECTION_FLAGS(0u32);

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    1usize,
                    Memory::PAGE_EXECUTE_READWRITE,
                    &mut old_protection,
                );

                Debug::WriteProcessMemory(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    [Self::NMSR_DATA_OLD.2].as_ptr() as _,
                    mem::size_of_val(&Self::NMSR_DATA_OLD.2),
                    None,
                );

                Memory::VirtualProtectEx(
                    handler.handle,
                    (handler.base_address + Self::NMSR_DATA_OLD.0) as _,
                    1usize,
                    old_protection,
                    &mut old_protection,
                );
            },
        }
    }
}

pub struct EsiFilter();

impl EsiFilter {
    pub const ESI_FILTER_DATA_0: (usize, [u8; 6usize], [u8; 6usize]) = (
        0x3ea873usize,
        [0xe9u8, 0x88u8, 0x57u8, 0xa1u8, 0xffu8, 0x90u8],
        [0x41u8, 0x8bu8, 0x01u8, 0x83u8, 0xf8u8, 0x07u8],
    );

    pub const ESI_FILTER_DATA_1: [u8; 543usize] = [
        0x0fu8, 0x87u8, 0x63u8, 0xacu8, 0x5eu8, 0x00u8, 0x48u8, 0x89u8, 0x1du8, 0xf3u8, 0x0fu8,
        0x00u8, 0x00u8, 0x48u8, 0x89u8, 0x0du8, 0xf4u8, 0x0fu8, 0x00u8, 0x00u8, 0x48u8, 0x89u8,
        0x15u8, 0xf5u8, 0x0fu8, 0x00u8, 0x00u8, 0x48u8, 0x89u8, 0x3du8, 0xf6u8, 0x0fu8, 0x00u8,
        0x00u8, 0x4cu8, 0x89u8, 0x05u8, 0xf7u8, 0x0fu8, 0x00u8, 0x00u8, 0x4cu8, 0x89u8, 0x0du8,
        0xf8u8, 0x0fu8, 0x00u8, 0x00u8, 0x4cu8, 0x89u8, 0x15u8, 0xf9u8, 0x0fu8, 0x00u8, 0x00u8,
        0x4cu8, 0x89u8, 0x25u8, 0xfau8, 0x0fu8, 0x00u8, 0x00u8, 0x49u8, 0x8bu8, 0xd8u8, 0x49u8,
        0x8bu8, 0xc8u8, 0xe8u8, 0xf7u8, 0xe9u8, 0x69u8, 0x00u8, 0x48u8, 0x8bu8, 0xcbu8, 0x44u8,
        0x0fu8, 0x28u8, 0xc0u8, 0xe8u8, 0xabu8, 0x37u8, 0x6au8, 0x00u8, 0x44u8, 0x0fu8, 0x28u8,
        0xc8u8, 0xe8u8, 0xe2u8, 0xe9u8, 0x69u8, 0x00u8, 0xf3u8, 0x0fu8, 0x10u8, 0x3du8, 0x96u8,
        0xbeu8, 0x89u8, 0x00u8, 0x48u8, 0x8bu8, 0xcbu8, 0xf3u8, 0x0fu8, 0x10u8, 0xb3u8, 0xf8u8,
        0x11u8, 0x00u8, 0x00u8, 0xf3u8, 0x0fu8, 0x5eu8, 0xc7u8, 0xf3u8, 0x0fu8, 0x59u8, 0xc0u8,
        0xf3u8, 0x0fu8, 0x5eu8, 0xf0u8, 0xf3u8, 0x0fu8, 0x59u8, 0x35u8, 0xd7u8, 0xb7u8, 0x89u8,
        0x00u8, 0xe8u8, 0xb6u8, 0xe9u8, 0x69u8, 0x00u8, 0xf3u8, 0x0fu8, 0x58u8, 0xf6u8, 0xf3u8,
        0x0fu8, 0x59u8, 0xc6u8, 0xf3u8, 0x0fu8, 0x59u8, 0x05u8, 0x92u8, 0xbdu8, 0x89u8, 0x00u8,
        0xe8u8, 0xa1u8, 0xa2u8, 0x7au8, 0x00u8, 0xf3u8, 0x44u8, 0x0fu8, 0x10u8, 0x15u8, 0x44u8,
        0xaau8, 0x89u8, 0x00u8, 0x41u8, 0x0fu8, 0x28u8, 0xc8u8, 0xf3u8, 0x44u8, 0x0fu8, 0x10u8,
        0x9bu8, 0x48u8, 0x12u8, 0x00u8, 0x00u8, 0xf3u8, 0x0fu8, 0x5cu8, 0xcfu8, 0x0fu8, 0x28u8,
        0xf0u8, 0xf3u8, 0x44u8, 0x0fu8, 0x58u8, 0xc7u8, 0xf2u8, 0x0fu8, 0x10u8, 0x3du8, 0x57u8,
        0xd0u8, 0x89u8, 0x00u8, 0x41u8, 0x0fu8, 0x28u8, 0xc2u8, 0xf3u8, 0x0fu8, 0x59u8, 0x35u8,
        0x1bu8, 0xa3u8, 0x89u8, 0x00u8, 0xf3u8, 0x41u8, 0x0fu8, 0x5eu8, 0xc8u8, 0xf3u8, 0x0fu8,
        0x5au8, 0xc9u8, 0x0fu8, 0x54u8, 0xcfu8, 0x66u8, 0x0fu8, 0x5au8, 0xd1u8, 0xf3u8, 0x0fu8,
        0x10u8, 0x0du8, 0x6bu8, 0xa5u8, 0x89u8, 0x00u8, 0xf3u8, 0x0fu8, 0x5cu8, 0xc2u8, 0xe8u8,
        0xeau8, 0xa9u8, 0x35u8, 0x00u8, 0x44u8, 0x0fu8, 0x28u8, 0xc0u8, 0x41u8, 0x0fu8, 0x28u8,
        0xc9u8, 0xf3u8, 0x0fu8, 0x5cu8, 0x0du8, 0x12u8, 0xb6u8, 0x89u8, 0x00u8, 0xf3u8, 0x44u8,
        0x0fu8, 0x58u8, 0x0du8, 0x09u8, 0xb6u8, 0x89u8, 0x00u8, 0x0fu8, 0x57u8, 0xc0u8, 0xf3u8,
        0x41u8, 0x0fu8, 0x5eu8, 0xc9u8, 0xf3u8, 0x0fu8, 0x5au8, 0xc1u8, 0x0fu8, 0x54u8, 0xc7u8,
        0x66u8, 0x0fu8, 0x5au8, 0xc8u8, 0x41u8, 0x0fu8, 0x28u8, 0xc2u8, 0xf3u8, 0x0fu8, 0x5cu8,
        0xc1u8, 0xf3u8, 0x0fu8, 0x10u8, 0x0du8, 0xe6u8, 0xa5u8, 0x89u8, 0x00u8, 0xe8u8, 0xa9u8,
        0xa9u8, 0x35u8, 0x00u8, 0xf3u8, 0x0fu8, 0x10u8, 0x0du8, 0xddu8, 0x4cu8, 0xa3u8, 0x00u8,
        0x0fu8, 0x28u8, 0xd6u8, 0xf3u8, 0x0fu8, 0x5cu8, 0xd1u8, 0xf3u8, 0x44u8, 0x0fu8, 0x59u8,
        0xc0u8, 0x0fu8, 0x57u8, 0xc0u8, 0xf3u8, 0x0fu8, 0x58u8, 0xceu8, 0xf3u8, 0x0fu8, 0x5eu8,
        0xd1u8, 0xf3u8, 0x0fu8, 0x5au8, 0xc2u8, 0x0fu8, 0x54u8, 0xc7u8, 0x66u8, 0x0fu8, 0x5au8,
        0xc8u8, 0x41u8, 0x0fu8, 0x28u8, 0xc2u8, 0xf3u8, 0x0fu8, 0x5cu8, 0xc1u8, 0xf3u8, 0x0fu8,
        0x10u8, 0x0du8, 0x1bu8, 0xa5u8, 0x89u8, 0x00u8, 0xe8u8, 0x6au8, 0xa9u8, 0x35u8, 0x00u8,
        0x41u8, 0x0fu8, 0x28u8, 0xd3u8, 0xf3u8, 0x44u8, 0x0fu8, 0x59u8, 0xc0u8, 0xf3u8, 0x0fu8,
        0x5cu8, 0x15u8, 0x11u8, 0xbbu8, 0x89u8, 0x00u8, 0xf3u8, 0x44u8, 0x0fu8, 0x58u8, 0x1du8,
        0x08u8, 0xbbu8, 0x89u8, 0x00u8, 0xf3u8, 0x41u8, 0x0fu8, 0x5eu8, 0xd3u8, 0x0fu8, 0x5au8,
        0xcau8, 0x0fu8, 0x54u8, 0xcfu8, 0x66u8, 0x0fu8, 0x5au8, 0xd1u8, 0xf3u8, 0x0fu8, 0x10u8,
        0x0du8, 0xb1u8, 0xaau8, 0x89u8, 0x00u8, 0xf3u8, 0x44u8, 0x0fu8, 0x5cu8, 0xd2u8, 0x41u8,
        0x0fu8, 0x28u8, 0xc2u8, 0xe8u8, 0x2bu8, 0xa9u8, 0x35u8, 0x00u8, 0xf3u8, 0x44u8, 0x0fu8,
        0x59u8, 0xc0u8, 0x41u8, 0x0fu8, 0x28u8, 0xd0u8, 0x48u8, 0x8bu8, 0x1du8, 0x3bu8, 0x0eu8,
        0x00u8, 0x00u8, 0x48u8, 0x8bu8, 0x0du8, 0x3cu8, 0x0eu8, 0x00u8, 0x00u8, 0x48u8, 0x8bu8,
        0x15u8, 0x3du8, 0x0eu8, 0x00u8, 0x00u8, 0x48u8, 0x8bu8, 0x3du8, 0x3eu8, 0x0eu8, 0x00u8,
        0x00u8, 0x4cu8, 0x8bu8, 0x05u8, 0x3fu8, 0x0eu8, 0x00u8, 0x00u8, 0x4cu8, 0x8bu8, 0x0du8,
        0x40u8, 0x0eu8, 0x00u8, 0x00u8, 0x4cu8, 0x8bu8, 0x15u8, 0x41u8, 0x0eu8, 0x00u8, 0x00u8,
        0x4cu8, 0x8bu8, 0x25u8, 0x42u8, 0x0eu8, 0x00u8, 0x00u8, 0xf3u8, 0x0fu8, 0x10u8, 0x05u8,
        0x02u8, 0x03u8, 0x00u8, 0x00u8, 0x0fu8, 0x2fu8, 0xc2u8, 0x0fu8, 0x87u8, 0x62u8, 0xaau8,
        0x5eu8, 0x00u8, 0x0fu8, 0x2fu8, 0x15u8, 0xf6u8, 0x02u8, 0x00u8, 0x00u8, 0x0fu8, 0x87u8,
        0x55u8, 0xaau8, 0x5eu8, 0x00u8, 0x41u8, 0x8bu8, 0x01u8, 0x83u8, 0xf8u8, 0x07u8, 0xe9u8,
        0x5au8, 0xa6u8, 0x5eu8, 0x00u8,
    ];

    pub fn new(handler: &Handler) -> Self {
        unsafe {
            Memory::VirtualAllocEx(
                handler.handle,
                Some((handler.base_address - 0x200000usize) as _),
                8192usize,
                Memory::MEM_COMMIT | Memory::MEM_RESERVE,
                Memory::PAGE_EXECUTE_READWRITE,
            );
        };

        unsafe {
            Debug::WriteProcessMemory(
                handler.handle,
                (handler.base_address - 0x200000usize) as _,
                Self::ESI_FILTER_DATA_1.as_ptr() as _,
                mem::size_of_val(&Self::ESI_FILTER_DATA_1),
                None,
            );
        };

        Self {}
    }

    pub fn close(&self, handler: &Handler) {
        unsafe {
            Memory::VirtualFreeEx(
                handler.handle,
                (handler.base_address - 0x200000usize) as _,
                0usize,
                Memory::MEM_RELEASE,
            );
        };
    }

    pub fn enable(&self, settings: (f32, f32), handler: &Handler) {
        unsafe {
            let mut old_protection = Memory::PAGE_PROTECTION_FLAGS(0u32);

            Memory::VirtualProtectEx(
                handler.handle,
                (handler.base_address + Self::ESI_FILTER_DATA_0.0) as _,
                6usize,
                Memory::PAGE_EXECUTE_READWRITE,
                &mut old_protection,
            );

            Debug::WriteProcessMemory(
                handler.handle,
                (handler.base_address + Self::ESI_FILTER_DATA_0.0) as _,
                Self::ESI_FILTER_DATA_0.1.as_ptr() as _,
                mem::size_of_val(&Self::ESI_FILTER_DATA_0.1),
                None,
            );

            Memory::VirtualProtectEx(
                handler.handle,
                (handler.base_address + Self::ESI_FILTER_DATA_0.0) as _,
                6usize,
                old_protection,
                &mut old_protection,
            );

            // Min ESI
            Debug::WriteProcessMemory(
                handler.handle,
                (handler.base_address - 0x200000usize + 0x500usize) as _,
                settings.0.to_le_bytes().as_ptr() as _,
                4usize,
                None,
            );

            // Max ESI
            Debug::WriteProcessMemory(
                handler.handle,
                (handler.base_address - 0x200000usize + 0x504usize) as _,
                settings.1.to_le_bytes().as_ptr() as _,
                4usize,
                None,
            );
        };
    }

    pub fn disable(&self, handler: &Handler) {
        unsafe {
            let mut old_protection = Memory::PAGE_PROTECTION_FLAGS(0u32);

            Memory::VirtualProtectEx(
                handler.handle,
                (handler.base_address + Self::ESI_FILTER_DATA_0.0) as _,
                6usize,
                Memory::PAGE_EXECUTE_READWRITE,
                &mut old_protection,
            );

            Debug::WriteProcessMemory(
                handler.handle,
                (handler.base_address + Self::ESI_FILTER_DATA_0.0) as _,
                Self::ESI_FILTER_DATA_0.2.as_ptr() as _,
                mem::size_of_val(&Self::ESI_FILTER_DATA_0.2),
                None,
            );

            Memory::VirtualProtectEx(
                handler.handle,
                (handler.base_address + Self::ESI_FILTER_DATA_0.0) as _,
                6usize,
                old_protection,
                &mut old_protection,
            );
        };
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Reason {
    NotFound,
    FailedToOpen,
    WrongVersion,
    TooManyInstances,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Handler {
    pub reason: Option<Reason>,
    pub handle: Option<HANDLE>,
    pub pid: u32,
    pub base_address: usize,
}

impl Handler {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        if sys.processes_by_exact_name("speng-starb.exe").count() > 1usize {
            return Self {
                reason: Some(Reason::TooManyInstances),
                ..Default::default()
            };
        }

        let pid = match sys.processes_by_exact_name("SpaceEngine.exe").nth(0usize) {
            Some(ph) => ph.pid().as_u32(),
            None => {
                return Self {
                    reason: Some(Reason::NotFound),
                    ..Default::default()
                }
            }
        };

        let handle = unsafe {
            match Threading::OpenProcess(
                Threading::PROCESS_QUERY_INFORMATION
                    | Threading::PROCESS_VM_OPERATION
                    | Threading::PROCESS_VM_READ
                    | Threading::PROCESS_VM_WRITE,
                false,
                pid,
            ) {
                Ok(ph) => ph,
                Err(_) => {
                    return Self {
                        reason: Some(Reason::FailedToOpen),
                        ..Default::default()
                    }
                }
            }
        };

        let mut buffer = Vec::<HINSTANCE>::with_capacity(1usize);
        buffer.push(HINSTANCE(0isize));

        unsafe {
            ProcessStatus::K32EnumProcessModules(
                handle,
                buffer.as_ptr() as _,
                mem::size_of_val(&buffer) as _,
                &mut 0u32,
            )
        };

        // Convert HINSTANCE to usize
        let base_address = buffer[0usize].0 as usize;

        let buffer = [0u8; 64usize];
        let version = unsafe {
            Debug::ReadProcessMemory(
                handle,
                (base_address + 0x49f4a0usize) as _,
                buffer.as_ptr() as _,
                64usize,
                None,
            );

            buffer == VERSION_DATA
        };

        // if !version {
        //     return Self {
        //         reason: Some(Reason::WrongVersion),
        //         ..Default::default()
        //     };
        // }

        Self {
            reason: None,
            handle: Some(handle),
            pid,
            base_address,
            ..Default::default()
        }
    }

    pub fn close(&self) {
        unsafe { Foundation::CloseHandle(self.handle) };
    }

    pub fn still_open(&self) -> bool {
        let mut exit_code = 0u32;
        unsafe { Threading::GetExitCodeProcess(self.handle, &mut exit_code) };

        // I'm proud of this (:
        exit_code == 259u32
    }
}
