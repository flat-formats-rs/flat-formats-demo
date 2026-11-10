//! # `flatdemo-core`
//!
//! Universal tick-based game replay container with $O(1)$ arbitrary tick seeking.

pub mod header;
pub mod reader;
pub mod state;
pub mod writer;

pub use header::{
    ChannelType, FLATDEMO_MAGIC, FLATDEMO_VERSION, FlatDemoHeader, SeekBlockDesc,
    TICKS_PER_SEEK_BLOCK,
};
pub use reader::{FlatDemoError, FlatDemoView};
pub use state::{EntityTransformRecord, SubTickUserCmd};
pub use writer::FlatDemoBuilder;
