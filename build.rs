// Computes a short content hash of style.css and exposes it as the STYLE_HASH
// env var. Templates use it as `?v={{ layout.style_hash }}` on the stylesheet
// link to bust the browser cache when CSS changes. The rerun-if-changed line
// makes Cargo rebuild whenever the CSS is edited.

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

fn hash_file(path: &str) -> String {
    let bytes =
        std::fs::read(path).expect("static/css/style.css must exist for cache-busting hash");
    let mut hasher = DefaultHasher::new();
    hasher.write(&bytes);
    let hash = format!("{:016x}", hasher.finish());
    hash[..8].to_string()
}

fn main() {
    println!("cargo:rerun-if-changed=static/css/style.css");

    let style_hash = hash_file("static/css/style.css");

    println!("cargo:rustc-env=STYLE_HASH={style_hash}");
}
