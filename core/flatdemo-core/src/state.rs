//! FlatDemo Entity Transform and User Command representations.

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// 32-byte Compact Entity Transform (cache-line friendly, 2 per 64B cache line).
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct EntityTransformRecord {
    /// Entity index / slot number (offset 0..4)
    pub entity_id: u32,
    /// Position [X, Y, Z] in 32-bit world floats (offset 4..16)
    pub position: [f32; 3],
    /// Rotation angles [Pitch, Yaw, Roll] (offset 16..28)
    pub angles: [f32; 3],
    /// Entity animation sequence / active state index (offset 28..30)
    pub sequence: u16,
    /// Bitflags (on_ground, ducking, alive, invisible) (offset 30..32)
    pub flags: u16,
}

const _: () = assert!(core::mem::size_of::<EntityTransformRecord>() == 32);

/// 32-byte Sub-Tick User Command (supports 1000 Hz mouse inputs and sub-tick precision).
#[repr(C)]
#[derive(Debug, Clone, Copy, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct SubTickUserCmd {
    /// 64-bit button bitmask (+attack, +jump, +duck, +use, etc.) (offset 0..8)
    pub buttons: u64,
    /// Relative sub-tick timestamp (0.0 to 1.0 within the tick) (offset 8..12)
    pub sub_tick_fraction: f32,
    /// View angle pitch (offset 12..16)
    pub view_pitch: f32,
    /// View angle yaw (offset 16..20)
    pub view_yaw: f32,
    /// High-precision mouse movement delta X (offset 20..22)
    pub mouse_dx: i16,
    /// High-precision mouse movement delta Y (offset 22..24)
    pub mouse_dy: i16,
    /// Reserved padding to exactly 32 bytes (offset 24..32)
    pub _pad: [u8; 8],
}

const _: () = assert!(core::mem::size_of::<SubTickUserCmd>() == 32);
