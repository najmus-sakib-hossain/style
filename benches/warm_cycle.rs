use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use style::StyleEngine;

fn setup_engine() -> StyleEngine {
    StyleEngine::load_from_disk().expect("load style.bin")
}

// Simulate an add/remove cycle using incremental path expectations.
fn bench_add_remove_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_remove_cycle");
    let engine = setup_engine();
    let target = "p-4"; // a common utility

    group.bench_function("compute_css_add_remove", |b| {
        b.iter_batched(
            || (),
            |_| {
                // Add
                let add_css = engine.css_for_class(target);
                black_box(&add_css);
                // Remove (simulate by just ensuring lookup again; real removal path is in rebuild logic)
                let rem_css = engine.css_for_class(target);
                black_box(&rem_css);
            },
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(name = warm; config = Criterion::default(); targets = bench_add_remove_cycle);
criterion_main!(warm);
