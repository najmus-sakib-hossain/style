use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::sync::{Arc, Mutex};
use style::core::{AppState, rebuild_styles_from_bytes};

fn setup_state() -> Arc<Mutex<AppState>> {
    let css_out = style::core::output::CssOutput::open("target/bench.css").expect("css out");
    Arc::new(Mutex::new(AppState {
        html_hash: 0,
        class_ids: Vec::new(),
        tmp_ids: Vec::new(),
        css_out,
        last_css_hash: 0,
        css_buffer: Vec::with_capacity(8192),
        class_list_checksum: 0,
        css_index: ahash::AHashMap::with_capacity(256),
        utilities_offset: 0,
        offsets_by_id: Vec::new(),
    }))
}

fn bench_rebuild_small(c: &mut Criterion) {
    let state = setup_state();
    let mut html_a = b"<div class=\"p-4 text-lg flex items-center\"></div>".to_vec();
    let mut html_b = b"<div class=\"p-6 text-lg flex items-center\"></div>".to_vec();
    let mut flip = false;
    c.bench_function("rebuild_small_html", |b| {
        b.iter(|| {
            let html = if flip { &html_a } else { &html_b };
            flip = !flip;
            rebuild_styles_from_bytes(state.clone(), html, false).unwrap();
            black_box(html);
        });
    });
}

fn custom_criterion() -> Criterion {
    Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(300))
        .measurement_time(std::time::Duration::from_millis(700))
        .sample_size(50)
}

criterion_group! { name = e2e; config = custom_criterion(); targets = bench_rebuild_small }
criterion_main!(e2e);
