//! Adapter for parsing GoldSrc/Half-Life/CS 1.6 (.dem) replay files.

use flatdemo_core::{
    FlatDemoBuilder,
    state::{EntityTransformRecord, SubTickUserCmd},
};
use hldemo::{Demo, FrameData};

#[derive(Debug, thiserror::Error)]
pub enum GoldSrcDemoError {
    #[error("Failed to parse GoldSrc demo: {0}")]
    ParseError(String),
    #[error("Demo contains no frames")]
    EmptyDemo,
}

/// Ingests a raw GoldSrc (.dem) file buffer and converts it into FlatDemo (.fld).
pub fn convert_goldsrc_dem_to_fld(
    dem_bytes: &[u8],
    compress: bool,
) -> Result<Vec<u8>, GoldSrcDemoError> {
    let demo =
        Demo::parse(dem_bytes).map_err(|e| GoldSrcDemoError::ParseError(format!("{:?}", e)))?;

    // GoldSrc typical tickrate is 100 Hz
    let tickrate = 100u32;
    let mut builder = FlatDemoBuilder::new(tickrate);

    // Group frames into 64-tick seek blocks
    let mut block_buffer = Vec::new();
    let mut frame_count = 0;

    for entry in &demo.directory.entries {
        for frame in &entry.frames {
            if let FrameData::NetMsg(ref net_msg) = frame.data {
                frame_count += 1;

                // Transcode ref_params / usercmd into compact EntityTransformRecord & SubTickUserCmd
                let transform = EntityTransformRecord {
                    entity_id: 1, // Local player slot
                    position: [
                        net_msg.info.ref_params.vieworg[0],
                        net_msg.info.ref_params.vieworg[1],
                        net_msg.info.ref_params.vieworg[2],
                    ],
                    angles: [
                        net_msg.info.ref_params.viewangles[0],
                        net_msg.info.ref_params.viewangles[1],
                        net_msg.info.ref_params.viewangles[2],
                    ],
                    sequence: net_msg.info.viewmodel as u16,
                    flags: 1,
                };

                let usercmd = SubTickUserCmd {
                    sub_tick_fraction: 0.0,
                    buttons: net_msg.info.usercmd.buttons as u64,
                    mouse_dx: 0,
                    mouse_dy: 0,
                    view_pitch: net_msg.info.usercmd.viewangles[0],
                    view_yaw: net_msg.info.usercmd.viewangles[1],
                    _pad: [0; 8],
                };

                block_buffer.extend_from_slice(zerocopy::IntoBytes::as_bytes(&transform));
                block_buffer.extend_from_slice(zerocopy::IntoBytes::as_bytes(&usercmd));

                if frame_count % 64 == 0 {
                    builder.add_block(&block_buffer, 0b11, compress);
                    block_buffer.clear();
                }
            }
        }
    }

    if !block_buffer.is_empty() {
        builder.add_block(&block_buffer, 0b11, compress);
    }

    if frame_count == 0 {
        // Fallback for empty demo: create at least 1 block
        let dummy = [0u8; 64];
        builder.add_block(&dummy, 1, compress);
    }

    Ok(builder.build())
}
