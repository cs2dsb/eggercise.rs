use std::{env, fs::{copy, create_dir_all, read_to_string, remove_file, File, OpenOptions}, io::{self, Read, Write}, path::PathBuf, process::Command, time::Instant};

use anyhow::bail;
use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use glob::glob;
use sha2::{ Sha384, Digest };
use shared::{ get_service_worker_info, HashedFile, ServiceWorkerPackage, WorkerInfo, SERVICE_WORKER_PACKAGE_FILENAME };
use wasm_opt::{
    OptimizationOptions,
    Pass,
};

const WASM_DEV_PROFILE: &str = "dev-min-size";
const USE_DEV_PROFILE: bool = false;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1m   {}", format!($($tokens)*))
    }
}

fn main() -> Result<(), anyhow::Error> {
    println!("cargo:rerun-if-changed=../service-worker");
    println!("cargo:rerun-if-changed=assets");
    
    let is_release_build = !cfg!(debug_assertions);

    let server_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let wasm_dir = out_dir.join("wasm");
    let assets_dir = server_dir
        .join("assets");
    let server_wasm_dir = assets_dir
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
        false => match USE_DEV_PROFILE {
            true => WASM_DEV_PROFILE,
            false => "debug",
        },
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
    } else if USE_DEV_PROFILE {
        // Apply the custom profile to the wasm build
        cargo_cmd.args([
            "--profile", WASM_DEV_PROFILE,
        ]);
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
    p!("Generating bindings for service-worker wasm took: {:.2}s", start.elapsed().as_secs_f32());

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
    let mut opt_options = OptimizationOptions::new_optimize_for_size_aggressively();
    if USE_DEV_PROFILE {
        opt_options.passes.more_passes.push(Pass::StripDwarf);
    }
    opt_options.run(&bg_lib_file, &wasm_opt_out)?;

    // Copy the output to the assets dir
    create_dir_all(&server_wasm_dir)?;
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

    // Prepare the package version info
    {
        let package_file_path = server_wasm_dir.join(SERVICE_WORKER_PACKAGE_FILENAME);

        if package_file_path.exists() {
            remove_file(&package_file_path)?;
        }

        let mut files = Vec::new();

        for f in 
            glob(&format!("{}/**/*", assets_dir.to_str().expect("Invalid assets_dir path")))? 
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|f| f.is_file())
        {
            let mut hasher = Sha384::new();
            let mut file = File::open(&f)?;

            io::copy(&mut file, &mut hasher)?;

            let hash_bytes = hasher.finalize();
            let hash = format!("sha384-{}", Base64Display::new(&hash_bytes, &STANDARD));

            let path = format!("/{}", f
                .strip_prefix(&assets_dir)?
                .to_str().expect(&format!("Invalid assets path ({:?})", &f)));

            files.push(HashedFile {
                path,
                hash,
            });
        }

        let mut package_out = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(package_file_path)?;

        let package = ServiceWorkerPackage {
            version: worker_version,
            files,
        };

        serde_json::to_writer_pretty(&mut package_out, &package)?;
    }
    
    Ok(())
}
