use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use muninn_core::classify::classify_by_extension;

fn sample_paths() -> Vec<PathBuf> {
    // Weighted mix — common office/text extensions dominate, with a long
    // tail of images, media, binaries, archives, and unknowns.
    let weighted: &[(&str, usize)] = &[
        ("pdf", 180),
        ("docx", 150),
        ("xlsx", 120),
        ("pptx", 60),
        ("txt", 100),
        ("csv", 80),
        ("json", 60),
        ("html", 50),
        ("png", 60),
        ("jpg", 60),
        ("mp4", 25),
        ("exe", 20),
        ("zip", 20),
        ("xyz", 15), // unknown extension
    ];

    let mut paths = Vec::with_capacity(1000);
    let mut idx = 0usize;
    while paths.len() < 1000 {
        let (ext, count) = weighted[idx % weighted.len()];
        for i in 0..count {
            if paths.len() >= 1000 {
                break;
            }
            paths.push(PathBuf::from(format!("dir/file_{i}.{ext}")));
        }
        idx += 1;
    }
    paths
}

fn bench_classify(c: &mut Criterion) {
    let paths = sample_paths();
    c.bench_function("classify_by_extension_x1000", |b| {
        b.iter(|| {
            for path in &paths {
                let _ = black_box(classify_by_extension(black_box(path)));
            }
        });
    });
}

criterion_group!(benches, bench_classify);
criterion_main!(benches);
