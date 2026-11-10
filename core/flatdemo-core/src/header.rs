//! FlatDemo (.fld) Header and Seek Table Binary Specification.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub const FLATDEMO_MAGIC: [u8; 4] = *b"FLD\0";
pub const FLATDEMO_VERSION: u32 = 1;
pub const TICKS_PER_SEEK_BLOCK: u32 = 64;

/// 64-byte cache-line sized FlatDemo container header (zero padding holes).
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct FlatDemoHeader {
    /// File signature b"FLD\0" (offset 0..4)
    pub magic: [u8; 4],
    /// Container format version (1) (offset 4..8)
    pub version: u32,
    /// Simulation tickrate in Hz (e.g. 64, 100, 128) (offset 8..12)
    pub tickrate: u32,
    /// Total seek blocks in stream (offset 12..16)
    pub block_count: u32,
    /// Total simulated ticks in replay (offset 16..24)
    pub total_ticks: u64,
    /// Offset to Seek Table in footer (offset 24..32)
    pub seek_table_offset: u64,
    /// BLAKE3 Merkle Root checksum of all seek blocks (offset 32..48)
    pub merkle_root_128: [u8; 16],
    /// Reserved space for flags and future format extensions (offset 48..64)
    pub _reserved: [u8; 16],
}

const _: () = assert!(core::mem::size_of::<FlatDemoHeader>() == 64);

/// 32-byte Seek Table Entry mapping ticks directly to byte ranges.
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct SeekBlockDesc {
    /// Starting tick of this block (multiple of 64) (offset 0..8)
    pub start_tick: u64,
    /// Absolute byte offset of block payload in container (offset 8..16)
    pub file_offset: u64,
    /// Compressed size on disk (offset 16..20)
    pub compressed_size: u32,
    /// Uncompressed memory footprint of the block (offset 20..24)
    pub uncompressed_size: u32,
    /// Bitmask of present channels in this block (offset 24..28)
    pub channel_mask: u32,
    /// Truncated block BLAKE3 checksum (offset 28..32)
    pub block_hash_32: [u8; 4],
}

const _: () = assert!(core::mem::size_of::<SeekBlockDesc>() == 32);

/// Standard Stream Channel Identifiers in FlatDemo.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ChannelType {
    /// Full keyframe snapshot of active entities.
    KeyframeSnapshot = 0,
    /// Columnar entity transform arrays (Pos X/Y/Z, Rot Pitch/Yaw/Roll).
    EntityTransforms = 1,
    /// User input commands (1000 Hz sub-tick mouse deltas, button bitmasks).
    UserCommands = 2,
    /// Discrete game events (Weapon fire, kills, bomb, radio).
    GameEvents = 3,
    /// Opus audio voice packets by entity/player slot.
    VoiceAudio = 4,
    /// Raw legacy engine packet passthrough.
    RawLegacyPassthrough = 5,
    /// Custom extension channel.
    Unknown = u32::MAX,
}

impl From<u32> for ChannelType {
    #[inline]
    fn from(val: u32) -> Self {
        match val {
            0 => Self::KeyframeSnapshot,
            1 => Self::EntityTransforms,
            2 => Self::UserCommands,
            3 => Self::GameEvents,
            4 => Self::VoiceAudio,
            5 => Self::RawLegacyPassthrough,
            _ => Self::Unknown,
        }
    }
}
