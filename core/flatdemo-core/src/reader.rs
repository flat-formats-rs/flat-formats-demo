//! Zero-allocation memory-mapped viewer and $O(1)$ seek engine for FlatDemo.

use zerocopy::FromBytes;

use crate::header::{
    FLATDEMO_MAGIC, FLATDEMO_VERSION, FlatDemoHeader, SeekBlockDesc, TICKS_PER_SEEK_BLOCK,
};

/// Errors encountered when inspecting or seeking in FlatDemo.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum FlatDemoError {
    #[error("Buffer too small: expected {expected}, got {actual}")]
    BufferTooSmall { expected: usize, actual: usize },
    #[error("Invalid magic: expected b\"FLD\\0\", got {0:?}")]
    InvalidMagic([u8; 4]),
    #[error("Unsupported format version: {0}")]
    UnsupportedVersion(u32),
    #[error("Seek table out of bounds: offset {offset} + size {size} > total {total}")]
    SeekTableOutOfBounds {
        offset: u64,
        size: usize,
        total: usize,
    },
    #[error("Requested tick {tick} exceeds total demo ticks {total_ticks}")]
    TickOutOfBounds { tick: u64, total_ticks: u64 },
    #[error("Decompression failed on block {block_idx}")]
    DecompressionFailed { block_idx: usize },
}

/// Zero-copy, zero-allocation memory-mapped viewer of a FlatDemo container.
#[derive(Debug, Clone, Copy)]
pub struct FlatDemoView<'a> {
    raw_bytes: &'a [u8],
    header: &'a FlatDemoHeader,
    seek_table: &'a [SeekBlockDesc],
}

impl<'a> FlatDemoView<'a> {
    /// Validates and constructs a FlatDemoView over an in-memory slice.
    pub fn new(bytes: &'a [u8]) -> Result<Self, FlatDemoError> {
        if bytes.len() < core::mem::size_of::<FlatDemoHeader>() {
            return Err(FlatDemoError::BufferTooSmall {
                expected: core::mem::size_of::<FlatDemoHeader>(),
                actual: bytes.len(),
            });
        }

        let (header, _) =
            FlatDemoHeader::ref_from_prefix(bytes).map_err(|_| FlatDemoError::BufferTooSmall {
                expected: core::mem::size_of::<FlatDemoHeader>(),
                actual: bytes.len(),
            })?;

        if header.magic != FLATDEMO_MAGIC {
            return Err(FlatDemoError::InvalidMagic(header.magic));
        }
        if header.version != FLATDEMO_VERSION {
            return Err(FlatDemoError::UnsupportedVersion(header.version));
        }

        let block_count = header.block_count as usize;
        let seek_table_size = block_count * core::mem::size_of::<SeekBlockDesc>();
        let table_start = header.seek_table_offset as usize;
        let table_end = table_start + seek_table_size;

        if table_end > bytes.len() {
            return Err(FlatDemoError::SeekTableOutOfBounds {
                offset: header.seek_table_offset,
                size: seek_table_size,
                total: bytes.len(),
            });
        }

        let (seek_table, _) = <[SeekBlockDesc]>::ref_from_prefix_with_elems(
            &bytes[table_start..table_end],
            block_count,
        )
        .map_err(|_| FlatDemoError::SeekTableOutOfBounds {
            offset: header.seek_table_offset,
            size: seek_table_size,
            total: bytes.len(),
        })?;

        Ok(Self {
            raw_bytes: bytes,
            header,
            seek_table,
        })
    }

    /// Accesses the container header.
    #[inline(always)]
    pub fn header(&self) -> &'a FlatDemoHeader {
        self.header
    }

    /// Returns the global seek table slice.
    #[inline(always)]
    pub fn seek_table(&self) -> &'a [SeekBlockDesc] {
        self.seek_table
    }

    /// Locates the seek block descriptor for a given target tick in $O(1)$ time.
    #[inline]
    pub fn seek_tick(
        &self,
        target_tick: u64,
    ) -> Result<(usize, &'a SeekBlockDesc, u32), FlatDemoError> {
        if target_tick >= self.header.total_ticks && self.header.total_ticks > 0 {
            return Err(FlatDemoError::TickOutOfBounds {
                tick: target_tick,
                total_ticks: self.header.total_ticks,
            });
        }

        let block_idx = (target_tick / (TICKS_PER_SEEK_BLOCK as u64)) as usize;
        if block_idx >= self.seek_table.len() {
            return Err(FlatDemoError::TickOutOfBounds {
                tick: target_tick,
                total_ticks: self.header.total_ticks,
            });
        }

        let block = &self.seek_table[block_idx];
        let sub_tick_offset = (target_tick % (TICKS_PER_SEEK_BLOCK as u64)) as u32;

        Ok((block_idx, block, sub_tick_offset))
    }

    /// Fetches the raw byte payload of a seek block.
    #[inline]
    pub fn block_payload(&self, block: &SeekBlockDesc) -> &'a [u8] {
        let start = block.file_offset as usize;
        let end = start + block.compressed_size as usize;
        &self.raw_bytes[start..end]
    }

    /// Decompresses block payload if compressed via Zstd.
    pub fn decompressed_block_payload(
        &self,
        block_idx: usize,
    ) -> Result<std::borrow::Cow<'a, [u8]>, FlatDemoError> {
        let block = &self.seek_table[block_idx];
        let raw = self.block_payload(block);

        #[cfg(feature = "zstd")]
        if block.compressed_size != block.uncompressed_size {
            let decompressed = zstd::decode_all(raw)
                .map_err(|_| FlatDemoError::DecompressionFailed { block_idx })?;
            return Ok(std::borrow::Cow::Owned(decompressed));
        }

        Ok(std::borrow::Cow::Borrowed(raw))
    }
}
