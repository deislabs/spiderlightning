use configs::*;
wit_bindgen_rust::import!("../../wit/configs.wit");
wit_error_rs::impl_error!(Error);

fn main() {
    let configs = Configs::open().expect("failed to open configs capability");
    assert!(configs.set("key", "value".as_bytes()).is_ok());
    assert!(configs.get("key").is_ok());
}
