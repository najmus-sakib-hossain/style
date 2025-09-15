use criterion::{Criterion, black_box, criterion_group, criterion_main};
use style::StyleEngine;

// Helper to construct a minimal engine/state for benchmarking.
fn setup_engine() -> StyleEngine {
    StyleEngine::load_from_disk().expect("load style.bin")
}

fn bench_single_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_class_update");
    let engine = setup_engine();
    // Representative class names (tune as needed)
    let classes = ["bg-red-500", "p-4", "text-lg", "flex", "items-center"];

    group.bench_function("compute_css_add", |b| {
        let mut i = 0usize;
        b.iter(|| {
            let cls = classes[i % classes.len()];
            black_box(engine.css_for_class(cls));
            i += 1;
        })
    });

    group.finish();
}

criterion_group!(name = single; config = Criterion::default(); targets = bench_single_add);
criterion_main!(single);
