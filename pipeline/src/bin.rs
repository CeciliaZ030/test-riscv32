use anyhow::Result;
use cargo_metadata::{Artifact, Message, Metadata, Target};
use chrono::Local;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, Write};
use std::{
    collections::HashMap,
    env, fs,
    hash::Hash,
    io::BufReader,
    iter,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread,
};

use crate::utils::GuestBuilder;

// #[cfg(test)]
// mod tests;

mod utils;
mod risc0;

fn extract_path(line: &str) -> Option<PathBuf> {
    let re = Regex::new(r"\(([^)]+)\)").unwrap();
    re.captures(line)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .and_then(|s| Some(PathBuf::from(s)))
}

use utils::*;
use risc0::*;


fn sp1() {
    println!("Hello, world!");
    sp1_helper::build_program("../cunt");
    build_test("../cunt");

    let meta = parse_metadata("../cunt");
    let tests = meta.tests();
    let bins = meta.bins();
    let libs = meta.libs();
    [tests, bins, libs].iter().for_each(|ps| {
        let names = ps.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
        println!("{:?}\n", names);
    });

    let builder = GuestBuilder::new(&meta, "riscv32im-succinct-zkvm-elf", "succinct")
        .rust_flags(&[
            "passes=loweratomic",
            "link-arg=-Ttext=0x00200800",
            "panic=abort",
        ])
        .custom_args(&["--ignore-rust-version"]);
    let executor = builder.test_command("debug", Some(vec!["cunt", "my_bin1"]));

    println!("executor: {:?}", executor);

    let _ = executor.execute()
        .expect("Execution failed")
        .sp1_placement(&meta);

}

pub fn risc0() {

    let meta = parse_metadata("guest");
    let tests = meta.tests();
    let bins = meta.bins();
    let libs = meta.libs();
    [tests, bins, libs].iter().for_each(|ps| {
        let names = ps.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
        println!("{:?}\n", names);
    });

    let builder = GuestBuilder::new(&meta, "riscv32im-risc0-zkvm-elf", "risc0")
        .rust_flags(&[
            "passes=loweratomic",
            "link-arg=-Ttext=0x00200800",
            "link-arg=--fatal-warnings",
            "panic=abort",
        ]);
        // .cc_compiler(
        //     risc0_data()
        //         .unwrap()
        //         .join("cpp/bin/riscv32-unknown-elf-gcc")
        // )
        // .c_flags(&["-march=rv32im", "-nostdlib"]);
        // .custom_args(&["--ignore-rust-version"]);
    let executor = builder.build_command("debug", None);

    println!("executor: {:?}", executor);

    let _ = executor.execute()
        .expect("Execution failed")
        .risc0_placement(&meta,"src/guest_bin.rs");

}

#[derive(Debug)]
pub struct Executor {
    cmd: Command,
    artifacts: Vec<PathBuf>,
    test: bool,
}

impl Executor {
    pub fn execute(mut self) -> anyhow::Result<Self> {
        let mut child = self
            .cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = BufReader::new(child.stdout.take().unwrap());
        let stderr = BufReader::new(child.stderr.take().unwrap());

        let stdout_handle = thread::spawn(move || {
            stdout.lines().for_each(|line| {
                println!("[docker] {}", line.unwrap());
            });
        });
        stderr.lines().for_each(|line| {
            let line = line.unwrap();
            println!("[zkvm-stdout] {}", line);
            if self.test && line.contains("Executable unittests") {
                if let Some(test) = extract_path(&line) {
                    self.artifacts
                        .iter_mut()
                        .find(|a| file_name(&test).contains(&file_name(a.clone())))
                        .map(|a| *a = test)
                        .expect("Failed to find test artifact");
                }
            }
        });
        stdout_handle.join().unwrap();

        let result = child.wait()?;
        if !result.success() {
            // Error message is already printed by cargo
            std::process::exit(result.code().unwrap_or(1))
        }
        Ok(self)
    }

    pub fn sp1_placement(&self, meta: &Metadata) -> Result<()> {
        let parant = meta.target_directory.parent().unwrap();
        let dest = parant.join("elf");
        fs::create_dir_all(&dest)?;

        for src in &self.artifacts {
            let dest = dest.join(
                if self.test { format!("test-{}", file_name(&src)) } else { file_name(&src) }
            );
            fs::copy(parant.join(src.to_str().unwrap()), dest.clone())?;
            println!("Copied test elf from\n[{:?}]\nto\n[{:?}]", src, dest);
        }
        Ok(())
    }

    pub fn risc0_placement(&self, meta: &Metadata, dest: &str) -> Result<()> {
        let parant = meta.target_directory.parent().unwrap();
        let mut dest = File::create(&dest).unwrap();
        for src in &self.artifacts {
            let src_name = file_name(&src);
            println!("src: {:?}", src);
            let guest = GuestListEntry::build(
                &if self.test { format!("test-{}", src_name) } else { src_name }, 
                &parant.join(src.to_str().unwrap()).to_string()
            ).unwrap();
            dest.write_all(guest.codegen_consts().as_bytes())?;
            println!("Wrote from\n[{:?}]\nto\n[{:?}]", src, dest);
        }
        Ok(())
    }
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
    let manifest = std::path::Path::new(path).join("Cargo.toml");
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd
        .no_deps()
        .manifest_path(manifest)
        .exec()
        .unwrap()
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
fn execute_build_cmd(program_dir: &Path) -> Result<std::process::ExitStatus, std::io::Error> {
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
        ]);
    println!("*******cmd: {:?}", cmd);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
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

    let src_elf_path = metadata.target_directory.parent().unwrap().join(
        elf_paths
            .first()
            .expect("Failed to extract carge test elf path")
            .to_str()
            .unwrap(),
    );
    println!("src_elf_path: {:?}", src_elf_path);

    let mut dest_elf_path = metadata.target_directory.parent().unwrap().join("elf");
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

fn file_name(path: &PathBuf) -> String {
    String::from(path.file_name().unwrap().to_str().unwrap())
}
