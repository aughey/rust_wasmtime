use std::{error::Error, io::Write};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

fn main() -> Result<(), Box<dyn Error>> {
    // An engine stores and configures global compilation settings like
    // optimization level, enabled wasm features, etc.
    let engine = Engine::default();

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()
        .unwrap()
        .build();

    let mut linker = Linker::new(&engine);

    wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

    for ret in 5.. {
        // A `Store` is what will own instances, functions, globals, etc. All wasm
        // items are stored within a `Store`, and it's what we'll always be using to
        // interact with the wasm world. Custom data can be stored in stores but for
        // now we just use `()`.
        println!("Creating store");
        let mut store = Store::new(&engine, wasi.clone());
        println!("Created store");

        let rust_source = format!(
            "
            #[no_mangle]
            pub extern \"C\" fn return_value() -> i32 {{
                {}
            }}
        ",
            ret
        );

        let temp_dir = tempfile::tempdir()?;

        // create a tempfile named 'lib.rs'
        let rust_file = temp_dir.path().join("lib.rs");
        {
            // write the rust source to the tempfile
            let mut rust_file = std::fs::File::create(&rust_file)?;
            rust_file.write_all(rust_source.as_bytes())?;
        }

        let outwasm = temp_dir.path().join("lib.wasm");

        // run rustc -O --crate-type cdynlib tempfile --target wasm32-wasi
        let rustc_output = std::process::Command::new("rustc")
            .arg("-O")
            .arg("--crate-type=cdylib")
            .arg(rust_file.as_path().to_str().unwrap())
            .arg("--target=wasm32-wasi")
            .arg("-o")
            .arg(outwasm.as_path().to_str().unwrap())
            .output()?;
        println!("rustc_output: {:?}", rustc_output);

        println!("out_wasm = {:?}", outwasm.as_path().to_str().unwrap());

        // We start off by creating a `Module` which represents a compiled form
        // of our input wasm module. In this case it'll be JIT-compiled after
        // we parse the text format.
        //const WASM_FILE: &str = "../target/wasm32-wasi/debug/wasmlib.wasm";
        println!("Loading wasm file: {}", outwasm.as_path().to_str().unwrap());
        let module = Module::from_file(&engine, outwasm.as_path().to_str().unwrap())?;
        println!("Loaded");

        println!("Creating instance");
        let link = linker.instantiate(&mut store, &module)?;
        println!("Created instance");

        let four_fn = link.get_typed_func::<(), i32>(&mut store, "return_value")?;
        let res = four_fn.call(&mut store, ())?;
        println!("{} == {}", ret, res);
    }

    // The `Instance` gives us access to various exported fun
    Ok(())
}
