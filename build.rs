const WIT_DIRECTORY: &str = "wit/*";

fn main() {
    println!("cargo:rerun-if-changed={}", WIT_DIRECTORY);
}