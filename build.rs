use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

fn hash_file(path: &str) -> String {
    let bytes = std::fs::read(path).unwrap_or_default();
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
