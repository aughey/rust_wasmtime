use std::error::Error;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // An engine stores and configures global compilation settings like
    // optimization level, enabled wasm features, etc.
    let engine = Engine::default();

    // We start off by creating a `Module` which represents a compiled form
    // of our input wasm module. In this case it'll be JIT-compiled after
    // we parse the text format.
    const WASM_FILE: &str = "../target/wasm32-wasi/debug/wasmlib.wasm";
    println!("Loading wasm file: {WASM_FILE}");
    let module = Module::from_file(&engine, WASM_FILE)?;
    println!("Loaded");

    for import in module.imports() {
        println!("Module Import: {:?}", import);
    }


    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

        let mut linker = Linker::new(&engine);

        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

    // A `Store` is what will own instances, functions, globals, etc. All wasm
    // items are stored within a `Store`, and it's what we'll always be using to
    // interact with the wasm world. Custom data can be stored in stores but for
    // now we just use `()`.
    println!("Creating store");
    let mut store = Store::new(&engine, wasi);
    println!("Created store");

    println!("Creating instance");
    let link = linker.instantiate(&mut store, &module)?;
    println!("Created instance");

    let four_fn = link.get_typed_func::<(), i32>(&mut store, "return_four")?;
    println!("{}", four_fn.call(&mut store, ()).unwrap());

    // The `Instance` gives us access to various exported fun
    Ok(())
}
