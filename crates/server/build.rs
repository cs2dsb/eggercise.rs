use std::{env, fs::{copy, create_dir_all, read_to_string, File, OpenOptions}, io::{Read, Write}, path::PathBuf, process::Command, time::Instant};

use anyhow::bail;
use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use shared::{ get_service_worker_info, WorkerInfo, SERVICE_WORKER_VERSION_FILENAME };
use wasm_opt::OptimizationOptions;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1m   {}", format!($($tokens)*))
    }
}

fn main() -> Result<(), anyhow::Error> {
    println!("cargo:rerun-if-changed=../service-worker");
    
    let is_release_build = !cfg!(debug_assertions);

    let server_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let wasm_dir = out_dir.join("wasm");
    let server_wasm_dir = server_dir
        .join("static")
        .join("wasm");

    let (worker_dir, worker_name, worker_version) = {
        let WorkerInfo { manifest_dir, name, version_with_timestamp, .. } = get_service_worker_info()?;
        (manifest_dir, name, version_with_timestamp)
    };

    let register_listeners_js = worker_dir.join("register_listeners.js");
    if !register_listeners_js.exists() {
        bail!("register_listeners.js missing, expected path: {:?}", register_listeners_js);
    }

    let profile = match is_release_build {
        true => "release",
        false => "debug",
    };
    let lib_file = wasm_dir
        .join("wasm32-unknown-unknown")
        .join(profile)
        .join(format!("{}.wasm", worker_name));
    
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.args([
        "rustc",
        "--package", "service-worker",
        "--lib",
        "--crate-type", "cdylib",
        "--target", "wasm32-unknown-unknown",
        "--target-dir", wasm_dir.to_str().expect("Invalid OUT_DIR path"),
    ]);

    if is_release_build {
        cargo_cmd.arg("--release");
    }
    
    let start = Instant::now();
    p!("Building service-worker wasm version {}", worker_version);
    assert!(cargo_cmd.status()?.success());
    p!("Building service-worker wasm took {:.2}s", start.elapsed().as_secs_f32());

    if !lib_file.exists() {
        bail!("Wasm file doesn't exist after running cargo build for worker. Should be at {:?}", lib_file);
    }

    let start = Instant::now();
    p!("Generating bindings for service-worker wasm");
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(&lib_file)
        .no_modules(true)?
        .remove_name_section(is_release_build)
        .remove_producers_section(is_release_build)
        .keep_debug(!is_release_build)
        .omit_default_module_path(false)
        .generate(lib_file.as_path().parent().unwrap())?;
    p!("Generating bindings for service-worker wasm took: {:2}s", start.elapsed().as_secs_f32());

    let js_file = lib_file.with_extension("js");
    let bg_lib_file = lib_file.with_file_name(format!("{}_bg.wasm", worker_name));

    // Check the output we were expecting was created
    if !js_file.exists() {        
        bail!("Bindings js file doesn't exist after running wasm-bindgen for worker. Should be at {:?}", js_file);
    }
    if !bg_lib_file.exists() {        
        bail!("Bingen lib file doesn't exist after running wasm-bindgen for worker. Should be at {:?}", bg_lib_file);
    }

    let js_out = server_wasm_dir.join(js_file.as_path().file_name().unwrap());
    let wasm_opt_out = lib_file.with_file_name(format!("{}_bg_opt.wasm", worker_name));

    // Optimize the wasm
    OptimizationOptions::new_optimize_for_size_aggressively()
        .run(&bg_lib_file, &wasm_opt_out)?;

    // Copy the output to the static dir
    create_dir_all(&server_wasm_dir)?;
    // These aren't needed with the wasm embedded in the js (see below)
    // copy(&bg_lib_file, server_wasm_dir.join(bg_lib_file.as_path().file_name().unwrap()))?;
    // copy(&wasm_opt_out, server_wasm_dir.join(wasm_opt_out.as_path().file_name().unwrap()))?;
    copy(&js_file, &js_out)?;

    {
        let wasm_bytes = {
            let mut file = File::open(&wasm_opt_out)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            bytes
        };
        let wasm_base64 = Base64Display::new(&wasm_bytes, &STANDARD).to_string();
        
        let snippet = read_to_string(&register_listeners_js)?
            // Embed the wasm as a base64 encoded string in the output js so that it is accessible
            // from the installed service worker without having to add extra cache logic in js
            .replace("SERVICE_WORKER_BASE64", &wasm_base64)
            // Include the version so the worker can work out if an update is needed
            .replace("SERVICE_WORKER_VERSION", &worker_version);

        let mut js_out = OpenOptions::new()
            .append(true)
            .open(&js_out)?;
        js_out.write_all(snippet.as_bytes())?;
    }

    {
        let mut version_out = OpenOptions::new()
            .write(true)
            .create(true)
            .open(server_wasm_dir.join(SERVICE_WORKER_VERSION_FILENAME))?;

        // Write the version out somewhere the server side can access it for update checks
        version_out.write_all(worker_version.as_bytes())?;
    }
    
    Ok(())
}
