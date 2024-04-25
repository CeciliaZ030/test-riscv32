use std::{collections::HashMap, env, fs, hash::Hash, io::BufReader, iter, path::{Path, PathBuf}, process::{Command, Stdio}, thread};
use std::io::BufRead;
use cargo_metadata::{Message, Metadata, Target};
use chrono::Local;
use regex::Regex;

// #[cfg(test)]
// mod tests;

mod utils;

fn extract_path(line: &str) -> Option<PathBuf> {
    let re = Regex::new(r"\(([^)]+)\)").unwrap();
    re
        .captures(line)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .and_then(|s| Some(PathBuf::from(s)))
}

fn main() {
    
    println!("Hello, world!");
    sp1_helper::build_program("../cunt");
    build_test("../cunt");
}

pub fn rerun_if_changed(paths: Vec<PathBuf>, env_vars: Vec<&str>) {
    // Only work in build.rs
    // Tell cargo to rerun the script only if program/{src, Cargo.toml, Cargo.lock} changes
    // Ref: https://doc.rust-lang.org/nightly/cargo/reference/build-scripts.html#rerun-if-changed
    for p in paths {
        println!("cargo::rerun-if-changed={}", p.display());
    }
    for v in env_vars {
        println!("cargo::rerun-if-env-changed={}", v);
    }
}

pub fn parse_metadata(path: &str) -> Metadata {
    let manifest =  std::path::Path::new(path).join("Cargo.toml");
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.manifest_path(manifest).exec().unwrap()
}


pub fn build_test(path: &str) {
    let program_dir = std::path::Path::new(path);

    let dirs = vec![
        program_dir.join("Cargo.toml"),
        program_dir.join("Cargo.lock"),
    ];
    rerun_if_changed(dirs, vec!["DUMMY______"]);

    
    let metadata = parse_metadata(path);
    let root_package = metadata.root_package();
    let root_package_name = root_package
        .as_ref()
        .map(|p| p.name.as_str())
        .unwrap_or("Program");
    println!(
        "cargo:warning={} built at {}",
        root_package_name,
        current_datetime()
    );


    execute_build_cmd(&program_dir).unwrap();
}

fn current_datetime() -> String {
    let now = Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}


/// Executes the `cargo prove build` command in the program directory
fn execute_build_cmd(
    program_dir: &Path,
) -> Result<std::process::ExitStatus, std::io::Error> {

    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.current_dir(program_dir);
    let metadata = metadata_cmd.exec().unwrap();
    let root_package = metadata.root_package();
    let root_package_name = root_package.as_ref().map(|p| &p.name);

    // println!("metadata: {:?}", metadata);
    println!("root_package: {:?}", root_package_name);
    
    let build_target = "riscv32im-succinct-zkvm-elf";
    let rust_flags = [
            "-C",
            "passes=loweratomic",
            "-C",
            "link-arg=-Ttext=0x00200800",
            "-C",
            "panic=abort",
        ];

    let mut cmd = Command::new("cargo");
    cmd.current_dir(program_dir)
        .env("RUSTUP_TOOLCHAIN", "succinct")
        .env("CARGO_MANIFEST_DIR", program_dir)
        .env("CARGO_ENCODED_RUSTFLAGS", rust_flags.join("\x1f"))
        .args([
            "test",
            "--release",
            "--target",
            build_target,
            "--locked",
            "--no-run",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn()?;

    let stdout = BufReader::new(child.stdout.take().unwrap());
    let stderr = BufReader::new(child.stderr.take().unwrap());


    let elf_paths = stderr
        .lines()
        .filter(|line| {
            println!("line: {:?}", line.as_ref().unwrap());
            line.as_ref()
                .is_ok_and(|l| l.contains("Executable unittests"))
        })
        .map(|line| extract_path(&line.unwrap()).unwrap())
        .collect::<Vec<_>>();
    println!("elf_paths: {:?}", elf_paths);
    
    let src_elf_path = metadata
        .target_directory
        .parent()
        .unwrap()
        .join(elf_paths.first().expect("Failed to extract carge test elf path").to_str().unwrap());
    println!("src_elf_path: {:?}", src_elf_path);

    let mut dest_elf_path = metadata.target_directory
        .parent()
        .unwrap()
        .join("elf");
    fs::create_dir_all(&dest_elf_path)?;
    dest_elf_path = dest_elf_path.join("riscv32im-succinct-zkvm-elf-test");
    println!("dest_elf_path: {:?}", dest_elf_path);

    fs::copy(&src_elf_path, &dest_elf_path)?;
    println!(
        "Copied test elf from\n[{:?}]\nto\n[{:?}]",
        src_elf_path, dest_elf_path
    );

    // Pipe stdout and stderr to the parent process with [sp1] prefix
    let stdout_handle = thread::spawn(move || {
        stdout.lines().for_each(|line| {
            println!("[sp1] {}", line.unwrap());
        });
    });


    // stderr.lines().for_each(|line| {
    //     let line = line.unwrap();
    //     if line.contains("Executable unittests") {
    //         let ep = extract_path( &line);
    //         println!("ep: {:?}", ep);
    //     }
    //     eprintln!("[sp1-err] {}", line);
    // });

    stdout_handle.join().unwrap();

    child.wait()
}

/// Build a [Command] with CARGO and RUSTUP_TOOLCHAIN environment variables
/// removed.
fn sanitized_cmd(tool: &str) -> Command {
    let mut cmd = Command::new(tool);
    for (key, _val) in env::vars().filter(|x| x.0.starts_with("CARGO")) {
        cmd.env_remove(key);
    }
    cmd.env_remove("RUSTUP_TOOLCHAIN");
    cmd
}