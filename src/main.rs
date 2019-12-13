use wasmer_runtime::{func, imports, instantiate};
use wasmer_wasi::{
    generate_import_object_for_version, WasiVersion
};
use grayarea::websocket_send_message;

fn main() {
    let path = std::env::args().nth(1).expect("USAGE: grayarea path/to/test.wasm");
    // Load the plugin data
    let wasm_bytes = std::fs::read(path.clone()).expect(&format!(
        "Could not read WASM plugin at {}",
        path
    ));

    // WASI imports
    let mut base_imports = generate_import_object_for_version(WasiVersion::Snapshot0, vec![], vec![], vec![], vec![(".".to_owned(), ".".into())]);
    // env is the default namespace for extern functions
    let custom_imports = imports! {
        "env" => {
            "websocket_send_message" => func!(websocket_send_message),
        },
    };
    // The WASI imports object contains all required import functions for a WASI module to run.
    // Extend this imports with our custom imports containing "it_works" function so that our custom wasm code may run.
    base_imports.extend(custom_imports);
    let mut instance =
        instantiate(&wasm_bytes[..], &base_imports).expect("failed to instantiate wasm module");

    // get a reference to the function "plugin_entrypoint"
    let entry_point = instance.func::<(), ()>("_start").unwrap();
    // call the "entry_point" function in WebAssembly
    let result = entry_point.call().expect("failed to execute module");
}
