extern crate autotools;

use autotools::Config;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use SHA256Status::{Mismatch, Unknown};

const PACKAGE_URL: &str =
    "https://github.com/cuddorg/cudd/releases/download/3.0.0/cudd-3.0.0.tar.gz";
const PACKAGE_SHA256: &str = "5fe145041c594689e6e7cf4cd623d5f2b7c36261708be8c9a72aed72cf67acce";
const CUDD_PATCH_PATH: &str = "patches/cudd-add-max-min.patch";

#[derive(Debug)]
#[allow(dead_code)]
enum FetchError {
    CommandError(std::process::ExitStatus),
    IOError(std::io::Error),
    PathExists,
}

enum SHA256Status {
    Match,
    Mismatch,
    Unknown,
}

impl From<std::io::Error> for FetchError {
    fn from(err: std::io::Error) -> FetchError {
        FetchError::IOError(err)
    }
}

fn run_command(cmd: &mut Command) -> Result<(String, String), FetchError> {
    let output = cmd.output()?;

    if output.status.success() {
        Ok((
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap(),
        ))
    } else {
        eprintln!("Command {:?} exited with status {}", cmd, output.status);
        Err(FetchError::CommandError(output.status))
    }
}

fn patch_cudd_sources(cudd_path: &Path) -> Result<(), String> {
    let patch_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(CUDD_PATCH_PATH);
    if !patch_path.exists() {
        return Err(format!(
            "CUDD patch file not found at {}",
            patch_path.display()
        ));
    }

    let output = Command::new("patch")
        .arg("--batch")
        .arg("--forward")
        .arg("-p1")
        .arg("-i")
        .arg(&patch_path)
        .current_dir(cudd_path)
        .output()
        .map_err(|e| format!("Failed to execute patch command: {e}"))?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Failed to apply CUDD patch {} in {} (status: {}).\nstdout:\n{}\nstderr:\n{}",
            patch_path.display(),
            cudd_path.display(),
            output.status,
            stdout,
            stderr
        ));
    }

    Ok(())
}

fn fetch_package(
    out_dir: &str,
    url: &str,
    sha256: &str,
) -> Result<(PathBuf, SHA256Status), FetchError> {
    let out_path = Path::new(out_dir);
    let target_path = out_path.join(Path::new(url).file_name().unwrap());
    let target_path_str = target_path.clone().into_os_string().into_string().unwrap();

    match target_path.metadata() {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            println!("Downloading {} to {}", url, target_path_str);
            let mut command = Command::new("curl");
            command.args(["-L", url, "-o", target_path_str.as_str()]);
            run_command(&mut command)?;
        }
        Ok(data) if data.is_file() => {
            println!("{} exists. Skipping download.", target_path_str);
        }
        Ok(_) => return Err(FetchError::PathExists),
        Err(error) => return Err(FetchError::from(error)),
    }

    let mut command_1 = Command::new("sha256sum");
    command_1.arg(target_path.clone());
    let mut command_2 = Command::new("shasum -a 256");
    command_2.arg(target_path.clone());
    let sha256_result = run_command(&mut command_1).or_else(|_| run_command(&mut command_2));

    let sha256_status = match sha256_result {
        Err(_) => SHA256Status::Unknown,
        Ok((output, _)) if output.contains(sha256) => SHA256Status::Match,
        _ => SHA256Status::Mismatch,
    };

    Ok((target_path, sha256_status))
}

fn main() -> Result<(), String> {
    let build_cudd = env::var_os("CARGO_FEATURE_BUILD_CUDD").is_some();
    if !build_cudd {
        return Ok(());
    }

    let out_dir = env::var("OUT_DIR")
        .map_err(|_| "Environmental variable `OUT_DIR` not defined.".to_string())?;

    let (tar_path, sha256_status) = fetch_package(&out_dir, PACKAGE_URL, PACKAGE_SHA256)
        .map_err(|e| format!("Error downloading CUDD package: {:?}.", e))?;
    let tar_path_str = tar_path.to_str().unwrap().to_string();

    match sha256_status {
        Unknown => eprintln!("WARNING: SHA256 not computed. Package validation skipped."),
        Mismatch => return Err("CUDD package SHA256 hash mismatch.".to_string()),
        _ => (),
    }

    let cudd_path = tar_path.with_extension("").with_extension("");
    let cudd_path_str = cudd_path.clone().into_os_string().into_string().unwrap();

    if !cudd_path.exists() {
        fs::create_dir_all(cudd_path.clone())
            .map_err(|e| format!("Cannot create CUDD directory: {:?}", e))?;
    }

    let mut tar_command = Command::new("tar");
    tar_command.args([
        "xf",
        &tar_path_str,
        "--strip-components=1",
        "-C",
        &cudd_path_str,
    ]);
    run_command(&mut tar_command).map_err(|e| format!("Error decompressing CUDD: {:?}", e))?;

    patch_cudd_sources(cudd_path.as_path())?;

    let build_output = Config::new(cudd_path).enable("dddmp", None).build();

    println!(
        "cargo:rustc-link-search=native={}",
        build_output.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=cudd");

    Ok(())
}
