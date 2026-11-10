use criterion::{Criterion, black_box, criterion_group, criterion_main};
use flatdemo_core::{
    FlatDemoBuilder, FlatDemoView,
    state::{EntityTransformRecord, SubTickUserCmd},
};

fn generate_sample_demo(blocks: usize) -> Vec<u8> {
    let mut builder = FlatDemoBuilder::new(100);
    for block_idx in 0..blocks {
        let mut block_data = Vec::new();
        for tick in 0..64 {
            let entity = EntityTransformRecord {
                entity_id: 1,
                position: [block_idx as f32, tick as f32, 0.0],
                angles: [0.0, 0.0, 0.0],
                sequence: 1,
                flags: 0,
            };
            let cmd = SubTickUserCmd {
                sub_tick_fraction: 0.0,
                buttons: 0,
                mouse_dx: 0,
                mouse_dy: 0,
                view_pitch: 0.0,
                view_yaw: 0.0,
                _pad: [0; 8],
            };
            block_data.extend_from_slice(zerocopy::IntoBytes::as_bytes(&entity));
            block_data.extend_from_slice(zerocopy::IntoBytes::as_bytes(&cmd));
        }
        builder.add_block(&block_data, 0b11, false);
    }
    builder.build()
}

fn bench_flatdemo(c: &mut Criterion) {
    let demo_bytes = generate_sample_demo(1000); // 64,000 ticks (~10.6 minutes of 100Hz gameplay)

    c.bench_function("flatdemo_view_mount_1000_blocks", |b| {
        b.iter(|| {
            let view = FlatDemoView::new(black_box(&demo_bytes)).unwrap();
            black_box(view);
        });
    });

    let view = FlatDemoView::new(&demo_bytes).unwrap();

    c.bench_function("flatdemo_o1_seek_arbitrary_tick", |b| {
        let mut tick = 0u64;
        b.iter(|| {
            tick = (tick + 37) % 64000;
            let res = view.seek_tick(black_box(tick)).unwrap();
            black_box(res);
        });
    });
}

criterion_group!(benches, bench_flatdemo);
criterion_main!(benches);
