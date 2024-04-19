use std::{env, fs::{copy, create_dir_all}, path::{Path, PathBuf}, process::Command, time::Instant};

use cargo_toml::Manifest;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1m   {}", format!($($tokens)*))
    }
}

fn read_manifest<P: AsRef<Path>>(path: P) -> Result<Manifest, anyhow::Error> {
    Ok(Manifest::from_path(path)?)
}

fn main() -> Result<(), anyhow::Error> {
    println!("cargo:rerun-if-changed=../service-worker");
    
    let is_release_build = !cfg!(debug_assertions);

    //OUT_DIR=/home/daniel/dev/eggercise.rs/target/debug/build/server-9002832088e59b55/out
    //CARGO_MANIFEST_DIR=/home/daniel/dev/eggercise.rs/crates/server
    let server_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let worker_dir = server_dir.join("../service-worker");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let wasm_dir = out_dir.join("wasm");
    let server_wasm_dir = server_dir
        .join("static")
        .join("wasm");

    let worker_manifest = read_manifest(worker_dir.join("Cargo.toml"))?;
    let worker_name = worker_manifest
        .package.ok_or(anyhow::anyhow!("Worker manifest missing package entry"))?
        .name
        .replace("-", "_");

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
    p!("Building service-worker wasm");
    assert!(cargo_cmd.status()?.success());
    p!("Building service-worker wasm took {:.2}s", start.elapsed().as_secs_f32());

    if !lib_file.exists() {
        anyhow::bail!("Wasm file doesn't exist after running cargo build for worker. Should be at {:?}", lib_file);
    }

    let start = Instant::now();
    p!("Generating bindings for service-worker wasm");
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(&lib_file)
        .web(true)?
        .remove_name_section(is_release_build)
        .remove_producers_section(is_release_build)
        .keep_debug(!is_release_build)
        .omit_default_module_path(false)
        .generate(lib_file.as_path().parent().unwrap())?;
    p!("Generating bindings for service-worker wasm took: {:2}s", start.elapsed().as_secs_f32());

    let js_file = lib_file.with_extension("js");
    let bg_lib_file = lib_file.with_file_name(format!("{}_bg.wasm", worker_name));

    if !js_file.exists() {        
        anyhow::bail!("Bindings js file doesn't exist after running wasm-bindgen for worker. Should be at {:?}", js_file);
    }
    if !bg_lib_file.exists() {        
        anyhow::bail!("Bingen lib file doesn't exist after running wasm-bindgen for worker. Should be at {:?}", bg_lib_file);
    }

    create_dir_all(&server_wasm_dir)?;
    copy(&bg_lib_file, server_wasm_dir.join(bg_lib_file.as_path().file_name().unwrap()))?;
    copy(&js_file, server_wasm_dir.join(js_file.as_path().file_name().unwrap()))?;
    
    Ok(())
}
