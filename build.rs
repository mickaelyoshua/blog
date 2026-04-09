use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

fn main() {
    println!("cargo:rerun-if-changed=static/css/style.css");
    println!("cargo:rerun-if-changed=static/vendor/");

    let css = std::fs::read("static/css/style.css").unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    hasher.write(&css);
    let hash = format!("{:016x}", hasher.finish());
    println!("cargo:rustc-env=STATIC_HASH={}", &hash[..8]);
}
