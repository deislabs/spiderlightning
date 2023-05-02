function _start() {
    let kv = keyvalue.open("placeholder-name");
    keyvalue.set(kv, "key", "Hello, JS Wasm!");
    console.log(fromUtf8(keyvalue.get(kv, "key")));
}