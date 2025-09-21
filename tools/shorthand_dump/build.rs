use std::path::Path;

fn main() {
    let schema = Path::new("../../.dx/style/style.fbs");
    println!("cargo:rerun-if-changed={}", schema.display());
    let out_dir = std::env::var("OUT_DIR").unwrap();
    flatc_rust::run(flatc_rust::Args {
        lang: "rust",
        inputs: &[schema],
        out_dir: Path::new(&out_dir),
        ..Default::default()
    })
    .expect("flatc schema compilation failed in shorthand_dump");
}
