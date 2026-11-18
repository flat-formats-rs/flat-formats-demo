use flatdemo_core::{
    FlatDemoBuilder, FlatDemoError, FlatDemoView,
    state::{EntityTransformRecord, SubTickUserCmd},
};

#[test]
fn test_flatdemo_roundtrip_and_o1_seek() {
    let tickrate = 128;
    let mut builder = FlatDemoBuilder::new(tickrate);

    // Build 10 blocks (640 ticks)
    for block_idx in 0..10 {
        let mut block_data = Vec::new();
        for tick in 0..64 {
            let entity = EntityTransformRecord {
                entity_id: 1,
                position: [block_idx as f32 * 100.0 + tick as f32, 0.0, 0.0],
                angles: [0.0, 0.0, 0.0],
                sequence: 1,
                flags: 0,
            };
            let cmd = SubTickUserCmd {
                sub_tick_fraction: 0.5,
                buttons: 1,
                mouse_dx: 5,
                mouse_dy: -2,
                view_pitch: 10.0,
                view_yaw: 90.0,
                _pad: [0; 8],
            };
            block_data.extend_from_slice(zerocopy::IntoBytes::as_bytes(&entity));
            block_data.extend_from_slice(zerocopy::IntoBytes::as_bytes(&cmd));
        }
        builder.add_block(&block_data, 0b11, true); // Compressed block
    }

    let fld_bytes = builder.build();
    let view = FlatDemoView::new(&fld_bytes).expect("Failed to mount FLD view");

    assert_eq!(view.header().tickrate, 128);
    assert_eq!(view.header().block_count, 10);
    assert_eq!(view.header().total_ticks, 640);
    assert_eq!(view.seek_table().len(), 10);

    // Test O(1) seek for various ticks
    let (idx0, desc0, sub0) = view.seek_tick(0).expect("Tick 0 seek failed");
    assert_eq!(idx0, 0);
    assert_eq!(desc0.start_tick, 0);
    assert_eq!(sub0, 0);

    let (idx350, desc350, sub350) = view.seek_tick(350).expect("Tick 350 seek failed");
    assert_eq!(idx350, 5);
    assert_eq!(desc350.start_tick, 320); // Block 5 starts at 5 * 64 = 320
    assert_eq!(sub350, 30);

    let (idx639, desc639, sub639) = view.seek_tick(639).expect("Tick 639 seek failed");
    assert_eq!(idx639, 9);
    assert_eq!(desc639.start_tick, 576); // Block 9 starts at 9 * 64 = 576
    assert_eq!(sub639, 63);

    // Out of bounds seek
    assert!(view.seek_tick(640).is_err());
    assert!(view.seek_tick(1000).is_err());

    // Decompress payload
    let decompressed = view
        .decompressed_block_payload(idx350)
        .expect("Decompression failed");
    assert_eq!(
        decompressed.len(),
        64 * (core::mem::size_of::<EntityTransformRecord>()
            + core::mem::size_of::<SubTickUserCmd>())
    );

    // Test try_block_payload
    let raw_payload = view
        .try_block_payload(desc350)
        .expect("Payload access failed");
    assert_eq!(raw_payload.len(), desc350.compressed_size as usize);

    // Test out of bounds decompressed_block_payload
    let err = view.decompressed_block_payload(99999).unwrap_err();
    assert!(matches!(err, FlatDemoError::BlockIndexOutOfBounds { .. }));
}

#[test]
fn test_flatdemo_corrupted_seek_table_bounds() {
    let tickrate = 128;
    let mut builder = FlatDemoBuilder::new(tickrate);
    let sample = [1u8; 100];
    builder.add_block(&sample, 0, false);
    let mut fld_bytes = builder.build();

    let view = FlatDemoView::new(&fld_bytes).expect("Valid view");
    let seek_table_offset = view.header().seek_table_offset as usize;
    // SeekBlockDesc layout: start_tick(u64), file_offset(u64), compressed_size(u32), ...
    // file_offset is at offset 8 within SeekBlockDesc in the seek table
    let offset_pos = seek_table_offset + 8;
    let bad_offset = 999999u64.to_le_bytes();
    fld_bytes[offset_pos..offset_pos + 8].copy_from_slice(&bad_offset);

    let view_res = FlatDemoView::new(&fld_bytes);
    assert!(matches!(
        view_res,
        Err(flatdemo_core::FlatDemoError::SeekBlockOutOfBounds { .. })
    ));
}
