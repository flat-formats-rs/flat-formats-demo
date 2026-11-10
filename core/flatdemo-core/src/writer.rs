//! FlatDemo (.fld) Container Builder.

use std::vec::Vec;
use zerocopy::IntoBytes;

use crate::header::{
    FLATDEMO_MAGIC, FLATDEMO_VERSION, FlatDemoHeader, SeekBlockDesc, TICKS_PER_SEEK_BLOCK,
};

/// Builder for constructing valid FlatDemo (.fld) containers with $O(1)$ seek tables.
#[derive(Debug)]
pub struct FlatDemoBuilder {
    tickrate: u32,
    blocks: Vec<(SeekBlockDesc, Vec<u8>)>,
    current_tick: u64,
}

impl FlatDemoBuilder {
    /// Creates a new builder with the simulation tickrate.
    pub fn new(tickrate: u32) -> Self {
        Self {
            tickrate,
            blocks: Vec::new(),
            current_tick: 0,
        }
    }

    /// Adds a 64-tick block payload with optional Zstd compression.
    pub fn add_block(&mut self, uncompressed_payload: &[u8], channel_mask: u32, compress: bool) {
        let uncompressed_size = uncompressed_payload.len() as u32;

        let (payload, compressed_size) = if compress {
            #[cfg(feature = "zstd")]
            {
                let comp = zstd::encode_all(uncompressed_payload, 3)
                    .unwrap_or_else(|_| uncompressed_payload.to_vec());
                let sz = comp.len() as u32;
                (comp, sz)
            }
            #[cfg(not(feature = "zstd"))]
            {
                (uncompressed_payload.to_vec(), uncompressed_size)
            }
        } else {
            (uncompressed_payload.to_vec(), uncompressed_size)
        };

        let block_hash = {
            #[cfg(feature = "blake3")]
            {
                let hash = blake3::hash(&payload);
                let mut truncated = [0u8; 4];
                truncated.copy_from_slice(&hash.as_bytes()[..4]);
                truncated
            }
            #[cfg(not(feature = "blake3"))]
            {
                [0u8; 4]
            }
        };

        let desc = SeekBlockDesc {
            start_tick: self.current_tick,
            file_offset: 0, // Assigned during build
            compressed_size,
            uncompressed_size,
            channel_mask,
            block_hash_32: block_hash,
        };

        self.blocks.push((desc, payload));
        self.current_tick += TICKS_PER_SEEK_BLOCK as u64;
    }

    /// Builds and serializes the complete container to a byte vector.
    pub fn build(mut self) -> Vec<u8> {
        let block_count = self.blocks.len() as u32;
        let mut current_offset = 64u64; // FlatDemoHeader size

        // Assign file offsets to each block (64B cache line aligned)
        for (desc, payload) in &mut self.blocks {
            desc.file_offset = current_offset;
            current_offset += payload.len() as u64;
            let align_pad = current_offset % 64;
            if align_pad != 0 {
                current_offset += 64 - align_pad;
            }
        }

        let seek_table_offset = current_offset;
        let seek_table_bytes_len = (block_count as usize) * core::mem::size_of::<SeekBlockDesc>();
        let total_size = (seek_table_offset as usize) + seek_table_bytes_len;

        let merkle_root_128 = {
            #[cfg(feature = "blake3")]
            {
                let mut hasher = blake3::Hasher::new();
                for (desc, _) in &self.blocks {
                    hasher.update(&desc.block_hash_32);
                }
                let mut truncated = [0u8; 16];
                truncated.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
                truncated
            }
            #[cfg(not(feature = "blake3"))]
            {
                [0u8; 16]
            }
        };

        let header = FlatDemoHeader {
            magic: FLATDEMO_MAGIC,
            version: FLATDEMO_VERSION,
            tickrate: self.tickrate,
            total_ticks: self.current_tick,
            block_count,
            seek_table_offset,
            merkle_root_128,
            _reserved: [0; 16],
        };

        let mut out = Vec::with_capacity(total_size);
        out.extend_from_slice(header.as_bytes());

        // Write block payloads
        for (desc, payload) in &self.blocks {
            let pad_needed = (desc.file_offset as usize).saturating_sub(out.len());
            out.resize(out.len() + pad_needed, 0);
            out.extend_from_slice(payload);
            let align_pad = out.len() % 64;
            if align_pad != 0 {
                out.resize(out.len() + (64 - align_pad), 0);
            }
        }

        // Write Seek Table at footer
        for (desc, _) in &self.blocks {
            out.extend_from_slice(desc.as_bytes());
        }

        out
    }
}
