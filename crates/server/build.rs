#![feature(path_file_prefix)]

use std::{
    env,
    fs::{copy, create_dir_all, read_to_string, remove_dir_all, remove_file, File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use anyhow::{bail, Context};
use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use chrono::Utc;
use glob::glob;
use sha2::{Digest, Sha384};
use shared::{
    get_client_info, get_server_info, get_service_worker_info, get_web_worker_info, CrateInfo, HashedFile, ServiceWorkerPackage, SERVICE_WORKER_PACKAGE_FILENAME
};
use wasm_opt::{OptimizationOptions, Pass};

const WASM_DEV_PROFILE: &str = "dev-min-size";
const USE_DEV_PROFILE: bool = true;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1m   {}", format!($($tokens)*))
    }
}

fn path_to_str<'a>(path: &'a Path) -> &'a str {
    path.to_str()
        .expect(&format!("Path \"{:?}\" cannot be converted to utf8", path))
}

fn path_prefix_to_str<'a>(path: &'a Path) -> &'a str {
    path.file_prefix()
        .expect(&format!("Path \"{:?}\" doesn't have a filename", path))
        .to_str()
        .expect(&format!("Path \"{:?}\" cannot be converted to utf8", path))
}

fn path_filename_to_str<'a>(path: &'a Path) -> &'a str {
    path.file_name()
        .expect(&format!("Path \"{:?}\" doesn't have a filename", path))
        .to_str()
        .expect(&format!("Path \"{:?}\" cannot be converted to utf8", path))
}

// Runs cargo rustc to build the wasm lib
fn build_wasm(package: &str, out_dir: &str, release: bool) -> Result<(), anyhow::Error> {
    let mut cargo_cmd = Command::new("cargo");
    cargo_cmd.env("BUILD_TIME", Utc::now().format("%Y%m%d %H%M%S").to_string());
    cargo_cmd.args([
        "rustc",
        "--package",
        package,
        "--lib",
        "--crate-type",
        "cdylib",
        "--target",
        "wasm32-unknown-unknown",
        "--target-dir",
        out_dir,
    ]);

    if release {
        cargo_cmd.arg("--release");
    } else if USE_DEV_PROFILE {
        // Apply the custom profile to the wasm build
        cargo_cmd.args(["--profile", WASM_DEV_PROFILE]);
    }

    let start = Instant::now();
    p!("Building {package} wasm");
    assert!(cargo_cmd.status()?.success());
    p!(
        "Building {package} wasm took {:.2}s",
        start.elapsed().as_secs_f32()
    );

    Ok(())
}

// Works out where the wasm is output to
fn wasm_out_path(lib_file_name: &str, wasm_dir: &Path, profile: &str) -> PathBuf {
    wasm_dir
        .join("wasm32-unknown-unknown")
        .join(profile)
        .join(format!("{lib_file_name}.wasm"))
}

// Run bindgen on the wasm lib to create the bg version + the js
fn generate_bindings(
    package: &str,
    input: &Path,
    release: bool,
    use_modules: bool,
) -> Result<(PathBuf, PathBuf), anyhow::Error> {
    if !input.exists() {
        bail!(
            "Wasm file doesn't exist after running cargo build for {package}. Should be at {:?}",
            input
        );
    }

    let start = Instant::now();
    p!("Generating bindings for {package} wasm");
    let mut bg = wasm_bindgen_cli_support::Bindgen::new();

    if use_modules {
        bg.web(true)?;
    } else {
        bg.no_modules(true)?;
    }

    bg.input_path(input)
        .remove_name_section(release)
        .remove_producers_section(release)
        .keep_debug(!release)
        .omit_default_module_path(false)
        .generate(input.parent().unwrap())?;

    p!(
        "Generating bindings for {package} wasm took: {:.2}s",
        start.elapsed().as_secs_f32()
    );

    let js_file = input.with_extension("js");
    let bg_file = input.with_file_name(format!("{}_bg.wasm", path_prefix_to_str(input)));

    // Check the output we were expecting was created
    if !js_file.exists() {
        bail!(
            "Bingen js file doesn't exist after running wasm-bindgen. Should be at {:?}",
            js_file
        );
    }
    if !bg_file.exists() {
        bail!(
            "Bingen lib file doesn't exist after running wasm-bindgen. Should be at {:?}",
            bg_file
        );
    }

    Ok((bg_file, js_file))
}

// Run wasm-opt on the wasm
fn optimize_wasm(input: &Path) -> Result<PathBuf, anyhow::Error> {
    let wasm_opt_out = input.with_file_name(format!("{}_opt.wasm", path_prefix_to_str(input)));

    // Optimize the wasm
    let mut opt_options = OptimizationOptions::new_optimize_for_size_aggressively();
    if USE_DEV_PROFILE {
        opt_options.passes.more_passes.push(Pass::StripDwarf);
    }
    opt_options.run(&input, &wasm_opt_out)?;

    // Check the output we were expecting was created
    if !wasm_opt_out.exists() {
        bail!(
            "Optimized bg file doesn't exist after running wasm-opt. Should be at {:?}",
            wasm_opt_out
        );
    }

    Ok(wasm_opt_out)
}

fn main() -> Result<(), anyhow::Error> {
    let client_info = get_client_info()?;
    let service_worker_info = get_service_worker_info()?;
    let web_worker_info = get_web_worker_info()?;
    let server_info = get_server_info()?;

    println!(
        "cargo:rerun-if-changed={}",
        path_to_str(&client_info.manifest_dir)
    );
    println!(
        "cargo:rerun-if-changed={}",
        path_to_str(&service_worker_info.manifest_dir)
    );
    println!(
        "cargo:rerun-if-changed={}",
        path_to_str(&web_worker_info.manifest_dir)
    );

    let is_release_build = !cfg!(debug_assertions);
    let server_dir = server_info.manifest_dir.clone();
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let wasm_dir = out_dir.join("wasm");
    let wasm_dir_str = path_to_str(&wasm_dir);
    let assets_dir = server_dir.join("assets");
    let server_wasm_dir = assets_dir.join("wasm");

    // Add everything in the assets folder *except* the wasm dir to rerun-if-changed
    // This won't work for new files in the root but this is an acceptable tradeoff
    // to prevent rebuilding every time the wasm folder is touched. The alternative
    // would be to diff the output of this build with the wasm folder and not
    // update it if it hasn't changed but this still requires running build.rs
    // every time
    for f in glob(&format!(
        "{}/**/*",
        assets_dir.to_str().expect("Invalid assets_dir path")
    ))?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .into_iter()
    .filter(|f| !f.starts_with(&server_wasm_dir))
    {
        println!("cargo:rerun-if-changed={}", path_to_str(&f));
    }

    p!("Out path: {:?}", out_dir);
    let CrateInfo {
        manifest_dir: service_worker_dir,
        lib_file_name: service_worker_lib_file_name,
        package_name: service_worker_package_name,
        version_with_timestamp: service_worker_version,
        ..
    } = service_worker_info;

    let CrateInfo {
        manifest_dir: web_worker_dir,
        lib_file_name: web_worker_lib_file_name,
        package_name: web_worker_package_name,
        version_with_timestamp: web_worker_version,
        ..
    } = web_worker_info;

    let CrateInfo {
        lib_file_name: client_lib_file_name,
        package_name: client_package_name,
        ..
    } = client_info;

    let service_worker_register_listeners_js = service_worker_dir.join("register_listeners.js");
    if !service_worker_register_listeners_js.exists() {
        bail!(
            "service worker register_listeners.js missing, expected path: {:?}",
            service_worker_register_listeners_js
        );
    }

    let web_worker_register_listeners_js = web_worker_dir.join("register_listeners.js");
    if !web_worker_register_listeners_js.exists() {
        bail!(
            "web worker register_listeners.js missing, expected path: {:?}",
            web_worker_register_listeners_js
        );
    }

    let profile = match is_release_build {
        true => "release",
        false => match USE_DEV_PROFILE {
            true => WASM_DEV_PROFILE,
            false => "debug",
        },
    };

    // worker can't use modules because browser support for modules in service
    // workers is minimal
    build_wasm(&service_worker_package_name, wasm_dir_str, is_release_build)
    .context("build_wasm[service_worker]")?;
    build_wasm(&web_worker_package_name, wasm_dir_str, is_release_build)
        .context("build_wasm[web_worker]")?;
    build_wasm(&client_package_name, wasm_dir_str, is_release_build)
        .context("build_wasm[client]")?;

    let service_worker_wasm_file = wasm_out_path(&service_worker_lib_file_name, &wasm_dir, profile);
    let web_worker_wasm_file = wasm_out_path(&web_worker_lib_file_name, &wasm_dir, profile);
    let client_wasm_file = wasm_out_path(&client_lib_file_name, &wasm_dir, profile);
    
    let (service_worker_bg_file, service_worker_js_file) = generate_bindings(
        &service_worker_lib_file_name,
        &service_worker_wasm_file,
        is_release_build,
        false,
    )
    .context("generate_bindings[service_worker]")?;
    
    let (web_worker_bg_file, web_worker_js_file) = generate_bindings(
        &web_worker_lib_file_name,
        &web_worker_wasm_file,
        is_release_build,
        false,
    )
    .context("generate_bindings[web_worker]")?;
    
    let (client_bg_file, client_js_file) = generate_bindings(
        &client_lib_file_name,
        &client_wasm_file,
        is_release_build,
        true,
    )
    .context("generate_bindings[client]")?;

    let service_worker_bg_opt_file = optimize_wasm(&service_worker_bg_file).context("optimize_wasm[service_worker]")?;
    let web_worker_bg_opt_file = optimize_wasm(&web_worker_bg_file).context("optimize_wasm[web_worker]")?;
    let client_bg_opt_file = optimize_wasm(&client_bg_file).context("optimize_wasm[client]")?;

    // Construct the output paths
    let service_worker_js_out = server_wasm_dir.join(path_filename_to_str(&service_worker_js_file));
    let web_worker_js_out = server_wasm_dir.join(path_filename_to_str(&web_worker_js_file));
    let client_js_out = server_wasm_dir.join(path_filename_to_str(&client_js_file));

    // Note this lops off the _opt which is necessary to restore the expected
    // bind-gen filename
    let client_wasm_out = server_wasm_dir.join(path_filename_to_str(&client_bg_file));

    // Delete the output directory if it exists
    if server_wasm_dir.exists() {
        remove_dir_all(&server_wasm_dir).context("remove_dir_all[server_wasm_dir]")?;
    }

    // Recreate it
    create_dir_all(&server_wasm_dir).context("create_dir_all[server_wasm_dir]")?;

    // Copy the output to the assets dir
    copy(&service_worker_js_file, &service_worker_js_out).context("copy[service_worker_js_file]")?;
    copy(&web_worker_js_file, &web_worker_js_out).context("copy[web_worker_js_file]")?;
    copy(&client_bg_opt_file, &client_wasm_out).context("copy[client_bg_opt_file]")?;
    copy(&client_js_file, &client_js_out).context("copy[client_js_file]")?;

    // TODO: this is a bit of a bodge
    for f in glob(&format!(
        "{}/**/sqlite3*",
        assets_dir.to_str().expect("Invalid assets_dir path")
    ))?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?
    .into_iter()
    .filter(|f| f.is_file())
    {
        copy(&f, server_wasm_dir.join(path_filename_to_str(&f)))?;
    }

    // Embed the wasm as a base64 encoded string in the output js so that it is
    // accessible from the installed service worker without having to add extra
    // cache logic in js
    {
        p!("Loading service worker wasm bytes");
        let service_worker_wasm_bytes = {
            let mut file = File::open(&service_worker_bg_opt_file)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            bytes
        };
        p!("Loading web worker wasm bytes");
        let web_worker_wasm_bytes = {
            let mut file = File::open(&web_worker_bg_opt_file)?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            bytes
        };

        p!("Encoding service worker wasm bytes in base64");
        let service_worker_wasm_base64 = Base64Display::new(&service_worker_wasm_bytes, &STANDARD).to_string();
        p!("Encoding web worker wasm bytes in base64");
        let web_worker_wasm_base64 = Base64Display::new(&web_worker_wasm_bytes, &STANDARD).to_string();

        p!("Reading service worker registration js");
        let service_worker_snippet = read_to_string(&service_worker_register_listeners_js)?
            .replace("SERVICE_WORKER_BASE64", &service_worker_wasm_base64)
            // Include the version so the worker can work out if an update is needed
            .replace("SERVICE_WORKER_VERSION", &service_worker_version);
        p!("Reading web worker registration js");
        let web_worker_snippet = read_to_string(&web_worker_register_listeners_js)?
            .replace("WEB_WORKER_BASE64", &web_worker_wasm_base64)
            // Include the version so the worker can work out if an update is needed
            .replace("WEB_WORKER_VERSION", &web_worker_version);

        p!(
            "Appending service worker registration and base64 wasm to {}",
            path_filename_to_str(&service_worker_js_out)
        );
        let mut service_worker_js_out = OpenOptions::new().append(true).open(&service_worker_js_out)?;
        service_worker_js_out.write_all(service_worker_snippet.as_bytes())?;
        p!(
            "Appending web worker registration and base64 wasm to {}",
            path_filename_to_str(&web_worker_js_out)
        );
        let mut web_worker_js_out = OpenOptions::new().append(true).open(&web_worker_js_out)?;
        web_worker_js_out.write_all(web_worker_snippet.as_bytes())?;
    }

    // Prepare the package version info
    {
        p!("Generating worker package json");
        let package_file_path = server_wasm_dir.join(SERVICE_WORKER_PACKAGE_FILENAME);

        if package_file_path.exists() {
            remove_file(&package_file_path)?;
        }

        let mut files = Vec::new();

        for f in glob(&format!(
            "{}/**/*",
            assets_dir.to_str().expect("Invalid assets_dir path")
        ))?
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|f| f.is_file())
        {
            p!("   Hashing {:?}", path_filename_to_str(&f));
            let mut hasher = Sha384::new();
            let mut file = File::open(&f)?;

            io::copy(&mut file, &mut hasher)?;

            let hash_bytes = hasher.finalize();
            let hash = format!("sha384-{}", Base64Display::new(&hash_bytes, &STANDARD));

            let path = format!(
                "/{}",
                f.strip_prefix(&assets_dir)?
                    .to_str()
                    .expect(&format!("Invalid assets path ({:?})", &f))
            );

            files.push(HashedFile {
                path,
                hash,
            });
        }

        p!("Saving worker package json");
        let mut package_out = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(package_file_path)?;

        let package = ServiceWorkerPackage {
            version: service_worker_version,
            files,
        };

        serde_json::to_writer_pretty(&mut package_out, &package)?;
    }

    p!("Done");

    Ok(())
}
