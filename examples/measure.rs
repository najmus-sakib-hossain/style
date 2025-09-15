use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use style::core::{AppState, rebuild_styles_from_bytes};

fn main() {
    let css_out = style::core::output::CssOutput::open("target/bench.css").expect("css out");
    let state = Arc::new(Mutex::new(AppState {
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
    }));

    let html_a = b"<div class=\"p-4 text-lg flex items-center\"></div>".to_vec();
    let html_b = b"<div class=\"p-6 text-lg flex items-center\"></div>".to_vec();

    // Warmup
    for _ in 0..50 {
        let _ = rebuild_styles_from_bytes(state.clone(), &html_a, true);
    }

    const ITERS: usize = 10_000;
    let mut flip = false;
    let start = Instant::now();
    for _ in 0..ITERS {
        let h = if flip { &html_a } else { &html_b };
        flip = !flip;
        rebuild_styles_from_bytes(state.clone(), h, false).unwrap();
    }
    let elapsed = start.elapsed();
    let per = elapsed.as_nanos() as f64 / ITERS as f64; // ns per iteration
    println!(
        "Total: {:?}  per-iter: {:.1} ns  (~{:.2} µs)",
        elapsed,
        per,
        per / 1000.0
    );

    // Print small summary of classes count
    let s = state.lock().unwrap();
    println!(
        "classes: {} bytes_css_buffer: {}",
        s.class_ids.len(),
        s.css_buffer.len()
    );
}
