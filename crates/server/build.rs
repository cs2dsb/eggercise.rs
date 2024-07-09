#![feature(path_file_prefix)]

use std::{
    env,
    fmt::Display,
    fs::{
        self, copy, create_dir_all, read_to_string, remove_dir_all, remove_file, File, OpenOptions,
    },
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Instant, SystemTime},
};

use anyhow::{bail, ensure, Context};
use base64::{display::Base64Display, engine::general_purpose::STANDARD};
use chrono::{DateTime, Utc};
use glob::{glob, Pattern};
use sha2::{Digest, Sha384};
use shared::{
    get_client_info, get_server_info, get_service_worker_info, CrateInfo, HashedFile,
    ServiceWorkerPackage, SERVICE_WORKER_PACKAGE_FILENAME,
};
use wasm_opt::{OptimizationOptions, Pass};

const WASM_PROFILE: &str = "wasm-release";
const PACKAGE_BLOCKLIST: &[&str] =
    &["**/*_input.css", "**/AUTHORS", "**/*.txt", "**/LICENSE", "**/NEWS", "**/README.md"];

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1m   {}", format!($($tokens)*))
    }
}

fn path_to_str<'a>(path: &'a Path) -> &'a str {
    path.to_str().expect(&format!("Path \"{:?}\" cannot be converted to utf8", path))
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

fn modificiation_date<P: AsRef<Path>>(path: P) -> Result<DateTime<Utc>, anyhow::Error> {
    let path = path.as_ref();

    let newest_file = if !path.exists() {
        None
    } else if path.is_file() {
        Some(path.metadata()?.modified()?)
    } else {
        glob(&format!("{}/**/*", path_to_str(path),))?
            .into_iter()
            .flat_map(|f| f.map(|f| f.metadata()))
            .flat_map(|f| f.map(|m| m.modified()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
    };

    let mod_date = newest_file.unwrap_or(SystemTime::now());

    Ok(mod_date.into())
}

fn build_css<'a>(
    infile: &'a Path,
    outfile: &'a Path,
    workspace_root: &'a Path,
) -> Result<(), anyhow::Error> {
    ensure!(infile.exists(), "Infile ({:?}) does not exist", infile);

    let in_modtime = modificiation_date(infile)?;
    let out_modtime = modificiation_date(outfile)?;

    if outfile.exists() && in_modtime < out_modtime {
        return Ok(());
    }

    if outfile.exists() {
        // Remove the output file to make sure it gets a new modificiation date
        // as postcss seems to only update the file if the output materially changes
        remove_file(outfile)?;
    }

    let mut cmd = Command::new("bash");
    cmd.current_dir(workspace_root);
    cmd.args(["-c", "scripts/install_npm_deps && scripts/build_css"]);

    let output = cmd.output()?;
    if !output.status.success() {
        let std_out = String::from_utf8_lossy(&output.stdout);
        let std_err = String::from_utf8_lossy(&output.stderr);
        bail!("Command failed: {:?}\n{}\n{}", output.status, std_out, std_err);
    }

    Ok(())
}

fn link_migrations<'a>(workspace_root: &'a Path, out_dir: &'a Path) -> Result<(), anyhow::Error> {
    let last_hash_file = out_dir.join("migrations_hash");
    let last_hash = if last_hash_file.is_file() {
        read_to_string(&last_hash_file).unwrap_or_default()
    } else {
        Default::default()
    };

    let mut cmd = Command::new("bash");
    cmd.current_dir(workspace_root);
    cmd.args([
        "-c",
        "find crates/shared/migrations -type f -print0 | sort -z | xargs -0 sha1sum | sha1sum",
    ]);

    let new_hash = time("Hashing migrations", 2, || run_cmd_and_log_errors(cmd))?;
    if last_hash != new_hash {
        fs::write(&last_hash_file, &new_hash)?;

        let mut cmd = Command::new("bash");
        cmd.current_dir(workspace_root);
        cmd.args(["-c", "scripts/link_migrations"]);

        run_cmd_and_log_errors(cmd)?;
    }

    Ok(())
}

fn run_cmd_and_log_errors(mut cmd: Command) -> Result<String, anyhow::Error> {
    let output = cmd.output()?;
    let std_out = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        let std_err = String::from_utf8_lossy(&output.stderr);
        bail!("Command failed: {:?}\n{}\n{}", output.status, std_out, std_err);
    }
    Ok(std_out.into())
}

// Runs cargo rustc to build the wasm lib
fn build_wasm(
    package: &str,
    out_dir: &str,
    modified_time: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let mut cmd = Command::new("cargo");
    cmd.env("BUILD_TIME", modified_time.format("%Y%m%d %H%M%S").to_string());
    cmd.args([
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
        "--profile",
        WASM_PROFILE,
    ]);

    let output = cmd.output()?;
    if !output.status.success() {
        let std_out = String::from_utf8_lossy(&output.stdout);
        let std_err = String::from_utf8_lossy(&output.stderr);
        bail!("Command failed: {:?}\n{}\n{}", output.status, std_out, std_err);
    }

    Ok(())
}

// Works out where the wasm is output to
fn wasm_out_path(lib_file_name: &str, wasm_dir: &Path, profile: &str) -> PathBuf {
    wasm_dir.join("wasm32-unknown-unknown").join(profile).join(format!("{lib_file_name}.wasm"))
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
    opt_options.passes.more_passes.push(Pass::StripDwarf);
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

fn time<F, R, S>(msg: S, indent: usize, f: F) -> Result<R, anyhow::Error>
where
    F: FnOnce() -> Result<R, anyhow::Error>,
    S: Display,
{
    let start = Instant::now();
    let r = f();
    let elapsed = start.elapsed().as_secs_f32();
    p!(
        "{}{:.2}s \"{msg}\" {}",
        "   ".repeat(indent),
        elapsed,
        if r.is_err() { "Error" } else { "" }
    );
    r
}

fn main() -> Result<(), anyhow::Error> {
    time("build.rs", 0, || {
        let package_blocklist = PACKAGE_BLOCKLIST
            .iter()
            .map(|pat| Pattern::new(pat))
            .collect::<Result<Vec<_>, _>>()
            .context("Converting PACKAGE_BLOCKLIST strings to Pattern")?;

        let client_info = get_client_info()?;
        let service_worker_info = get_service_worker_info()?;
        let server_info = get_server_info()?;

        let is_release_build = !cfg!(debug_assertions);
        let server_dir = server_info.manifest_dir.clone();
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        let wasm_dir = out_dir.join("wasm");
        let wasm_dir_str = path_to_str(&wasm_dir);
        let assets_dir = server_dir.join("assets");
        let server_wasm_dir = assets_dir.join("wasm");

        println!("cargo:rerun-if-changed={}", path_to_str(&client_info.manifest_dir));
        println!("cargo:rerun-if-changed={}", path_to_str(&service_worker_info.manifest_dir));

        p!("Out path: {:?}", out_dir);
        let CrateInfo {
            manifest_dir: service_worker_dir,
            lib_file_name: service_worker_lib_file_name,
            package_name: service_worker_package_name,
            version_with_timestamp: service_worker_version,
            ..
        } = service_worker_info;

        let CrateInfo {
            manifest_dir: client_dir,
            lib_file_name: client_lib_file_name,
            package_name: client_package_name,
            ..
        } = client_info;

        let in_css = assets_dir.join("css/main_input.css");
        let out_css = assets_dir.join("css/main_output.css");
        let workspace_root = service_worker_dir.join("../..");

        // Add everything in the assets folder *except* the wasm dir to rerun-if-changed
        // This won't work for new files in the root but this is an acceptable tradeoff
        // to prevent rebuilding every time the wasm folder is touched. The alternative
        // would be to diff the output of this build with the wasm folder and not
        // update it if it hasn't changed but this still requires running build.rs
        // every time
        // We monitor this to trigger creating the service worker package file. It's
        // unfortunate this restarts the server and does all the other build steps. It
        // could be split up down the line to reduce rework.
        for f in glob(&format!("{}/**/*", assets_dir.to_str().expect("Invalid assets_dir path")))?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|f| !f.starts_with(&server_wasm_dir) && !f.starts_with(&out_css))
        {
            // p!("{:?}", f);
            println!("cargo:rerun-if-changed={}", path_to_str(&f));
        }

        time("Linking migrations", 1, || link_migrations(&workspace_root, &out_dir))?;

        time("Building css", 1, || build_css(&in_css, &out_css, &workspace_root))?;

        let service_worker_register_listeners_js = service_worker_dir.join("register_listeners.js");
        if !service_worker_register_listeners_js.exists() {
            bail!(
                "service worker register_listeners.js missing, expected path: {:?}",
                service_worker_register_listeners_js
            );
        }

        let profile = WASM_PROFILE;

        let service_worker_mod_time = time("Find latest change for service worker", 1, || {
            modificiation_date(&service_worker_dir)
        })?;
        let client_mod_time =
            time("Find latest change for client", 1, || modificiation_date(&client_dir))?;

        // worker can't use modules because browser support for modules in service
        // workers is minimal
        time("Build service worker wasm", 1, || {
            build_wasm(&service_worker_package_name, wasm_dir_str, service_worker_mod_time)
                .context("build_wasm[service_worker]")
        })?;

        time("Build client wasm", 1, || {
            build_wasm(&client_package_name, wasm_dir_str, client_mod_time)
                .context("build_wasm[client]")
        })?;

        let service_worker_wasm_file =
            wasm_out_path(&service_worker_lib_file_name, &wasm_dir, profile);
        let client_wasm_file = wasm_out_path(&client_lib_file_name, &wasm_dir, profile);

        let (service_worker_bg_file, service_worker_js_file) =
            time("Generate service worker bindings", 1, || {
                generate_bindings(
                    &service_worker_lib_file_name,
                    &service_worker_wasm_file,
                    is_release_build,
                    false,
                )
                .context("generate_bindings[service_worker]")
            })?;

        let (client_bg_file, client_js_file) = time("Generate client bindings", 1, || {
            generate_bindings(&client_lib_file_name, &client_wasm_file, is_release_build, true)
                .context("generate_bindings[client]")
        })?;

        let (service_worker_bg_opt_file, client_bg_opt_file) = if is_release_build {
            let service_worker_bg_opt_file = time("Optimize service worker wasm", 1, || {
                optimize_wasm(&service_worker_bg_file).context("optimize_wasm[service_worker]")
            })?;
            let client_bg_opt_file = time("Optimize client wasm", 1, || {
                optimize_wasm(&client_bg_file).context("optimize_wasm[client]")
            })?;
            (service_worker_bg_opt_file, client_bg_opt_file)
        } else {
            (service_worker_bg_file.clone(), client_bg_file.clone())
        };

        // Construct the output paths
        let service_worker_js_out =
            server_wasm_dir.join(path_filename_to_str(&service_worker_js_file));
        let client_js_out = server_wasm_dir.join(path_filename_to_str(&client_js_file));

        // Note this lops off the _opt which is necessary to restore the expected
        // bind-gen filename
        let client_wasm_out = server_wasm_dir.join(path_filename_to_str(&client_bg_file));

        // Delete the output directory if it exists
        if server_wasm_dir.exists() {
            time("Remove output wasm dir", 1, || {
                remove_dir_all(&server_wasm_dir).context("remove_dir_all[server_wasm_dir]")
            })?;
        }

        // Recreate it
        create_dir_all(&server_wasm_dir).context("create_dir_all[server_wasm_dir]")?;

        // Copy the output to the assets dir
        time("Copy wasm & js files to output wasm dir", 1, || {
            copy(&service_worker_js_file, &service_worker_js_out)
                .context("copy[service_worker_js_file]")?;
            copy(&client_bg_opt_file, &client_wasm_out).context("copy[client_bg_opt_file]")?;
            copy(&client_js_file, &client_js_out).context("copy[client_js_file]")
        })?;

        // Embed the wasm as a base64 encoded string in the output js so that it is
        // accessible from the installed service worker without having to add extra
        // cache logic in js
        {
            let service_worker_wasm_bytes = time("Read service worker wasm bytes", 1, || {
                let mut file = File::open(&service_worker_bg_opt_file)?;
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                Ok(bytes)
            })?;

            let service_worker_wasm_base64 =
                time("Base64 encode service worker wasm bytes", 1, || {
                    Ok(Base64Display::new(&service_worker_wasm_bytes, &STANDARD).to_string())
                })?;

            let service_worker_snippet =
                time("Read service worker register listners and replace placeholders", 1, || {
                    Ok(read_to_string(&service_worker_register_listeners_js)?
                        .replace("SERVICE_WORKER_BASE64", &service_worker_wasm_base64)
                        // Include the version so the worker can work out if an update is
                        // needed
                        .replace("SERVICE_WORKER_VERSION", &service_worker_version))
                })?;

            time("Write service worker registration js", 1, move || {
                let mut service_worker_js_out =
                    OpenOptions::new().append(true).create(true).open(&service_worker_js_out)?;
                service_worker_js_out.write_all(service_worker_snippet.as_bytes())?;
                Ok(())
            })?;
        }

        // Prepare the package version info
        {
            let package_file_path = server_wasm_dir.join(SERVICE_WORKER_PACKAGE_FILENAME);

            if package_file_path.exists() {
                remove_file(&package_file_path)?;
            }

            time("Hashing files", 1, || {
                let mut files = Vec::new();

                for f in glob(&format!(
                    "{}/**/*",
                    assets_dir.to_str().expect("Invalid assets_dir path")
                ))?
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .filter(|f| f.is_file())
                // Filter out any that match the blocklist
                .filter(|f| !package_blocklist.iter().any(|p| p.matches(&path_to_str(f))))
                {
                    let hash = time(format!("Hashing {:?}", path_filename_to_str(&f)), 2, || {
                        let mut hasher = Sha384::new();
                        let mut file = File::open(&f)?;

                        io::copy(&mut file, &mut hasher)?;

                        let hash_bytes = hasher.finalize();
                        Ok(format!("sha384-{}", Base64Display::new(&hash_bytes, &STANDARD)))
                    })?;

                    let path = format!(
                        "/{}",
                        f.strip_prefix(&assets_dir)?
                            .to_str()
                            .expect(&format!("Invalid assets path ({:?})", &f))
                    );

                    files.push(HashedFile { path, hash });
                }

                let mut package_out = OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(package_file_path)?;

                let package = ServiceWorkerPackage { version: service_worker_version, files };

                serde_json::to_writer_pretty(&mut package_out, &package)?;
                Ok(())
            })?;
        }

        Ok(())
    })
}
