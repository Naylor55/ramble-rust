
use criterion::{criterion_group, criterion_main, Criterion};
use rust_rtmp_handler::media::video::H264Processor;

fn video_processing_benchmark(c: &mut Criterion) {
    let mut processor = H264Processor::new();
    let test_nalu = include_bytes!("../tests/data/test_nalu.h264");
    
    c.bench_function("h264_nalu_processing", |b| {
        b.iter(|| processor.process_nalu(test_nalu).unwrap())
    });
}

criterion_group!(benches, video_processing_benchmark);
criterion_main!(benches);