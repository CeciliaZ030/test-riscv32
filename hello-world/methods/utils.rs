use std::{
    borrow::Cow,
    collections::HashMap,
    default::Default,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use cargo_metadata::{Message, MetadataCommand, Package};
use serde::Deserialize;

pub const DIGEST_WORDS: usize = 8;

#[derive(Debug, Deserialize)]
pub struct Risc0Metadata {
    pub methods: Vec<String>,
}

impl Risc0Metadata {
    pub fn from_package(pkg: &Package) -> Option<Risc0Metadata> {
        let obj = pkg.metadata.get("risc0").unwrap();
        serde_json::from_value(obj.clone()).unwrap()
    }
}

/// Represents an item in the generated list of compiled guest binaries
#[derive(Debug, Clone)]
pub struct GuestListEntry {
    /// The name of the guest binary
    pub name: Cow<'static, str>,
    /// The compiled ELF guest binary
    pub elf: Cow<'static, [u8]>,
    /// The image id of the guest
    pub image_id: [u32; DIGEST_WORDS],
    /// The path to the ELF binary
    pub path: Cow<'static, str>,
}

impl GuestListEntry {
    /// Builds the [GuestListEntry] by reading the ELF from disk, and calculating the associated
    /// image ID.
    pub fn build(name: &str, elf_path: &str) -> Result<Self> {
        println!("*************  {} *************", elf_path);
        let elf = std::fs::read(elf_path)?;
        println!("******--*****  {} ******--*****", elf_path);

        // Todo(Cecilia)
        let image_id = [9u32; DIGEST_WORDS];

        Ok(Self {
            name: Cow::Owned(name.to_owned()),
            elf: Cow::Owned(elf),
            image_id,
            path: Cow::Owned(elf_path.to_owned()),
        })
    }

    pub fn codegen_consts(&self) -> String {
        // Quick check for '#' to avoid injection of arbitrary Rust code into the the
        // method.rs file. This would not be a serious issue since it would only
        // affect the user that set the path, but it's good to add a check.
        if self.path.contains('#') {
            panic!("method path cannot include #: {}", self.path);
        }

        let upper = self.name.to_uppercase().replace('-', "_");
        let mut parts: Vec<&str> = upper.split('_').collect();
        parts.pop();
        parts.push("TEST");
        let upper = parts.join("_");

        let image_id: [u32; DIGEST_WORDS] = self.image_id;
        let elf_path: &str = &self.path;
        let elf_contents: &[u8] = &self.elf;
        let f = format!(
            r##"
pub const {upper}_ELF: &[u8] = &{elf_contents:?};
pub const {upper}_ID: [u32; 8] = {image_id:?};
pub const {upper}_PATH: &str = r#"{elf_path}"#;
"##
        );
        f
    }
}

pub fn is_debug() -> bool {
    get_env_var("RISC0_BUILD_DEBUG") == "1"
}

pub fn get_env_var(name: &str) -> String {
    println!("cargo:rerun-if-env-changed={name}");
    env::var(name).unwrap_or_default()
}



/// Creates a std::process::Command to execute the given cargo
/// command in an environment suitable for targeting the zkvm guest.
pub fn cargo_command(subcmd: &str, rust_flags: &[&str]) -> Command {
    let rustc = sanitized_cmd("rustup")
        .args(["+risc0", "which", "rustc"])
        .output()
        .expect("rustup failed to find risc0 toolchain")
        .stdout;

    let rustc = String::from_utf8(rustc).unwrap();
    let rustc = rustc.trim();
    println!("Using rustc: {rustc}");

    let mut cmd = sanitized_cmd("cargo");
    let mut args = vec![subcmd, "--target", "riscv32im-risc0-zkvm-elf"];

    if std::env::var("RISC0_BUILD_LOCKED").is_ok() {
        args.push("--locked");
    }

    let rust_src = get_env_var("RISC0_RUST_SRC");
    if !rust_src.is_empty() {
        args.push("-Z");
        args.push("build-std=alloc,core,proc_macro,panic_abort,std");
        args.push("-Z");
        args.push("build-std-features=compiler-builtins-mem");
        cmd.env("__CARGO_TESTS_ONLY_SRC_ROOT", rust_src);
    }

    println!("Building guest package: cargo {}", args.join(" "));

    let rustflags_envvar = [
        rust_flags,
        &[
            // Replace atomic ops with nonatomic versions since the guest is single threaded.
            "-C",
            "passes=loweratomic",
            // Specify where to start loading the program in
            // memory.  The clang linker understands the same
            // command line arguments as the GNU linker does; see
            // https://ftp.gnu.org/old-gnu/Manuals/ld-2.9.1/html_mono/ld.html#SEC3
            // for details.
            "-C",
            &format!("link-arg=-Ttext=0x{:08X}", 0x0020_0800),
            // Apparently not having an entry point is only a linker warning(!), so
            // error out in this case.
            "-C",
            "link-arg=--fatal-warnings",
            "-C",
            "panic=abort",
        ],
    ]
    .concat()
    .join("\x1f");

    let cc_path = risc0_data()
        .unwrap()
        .join("cpp/bin/riscv32-unknown-elf-gcc");
    let c_flags = "-march=rv32im -nostdlib";
    cmd.env("RUSTC", rustc)
        .env("CARGO_ENCODED_RUSTFLAGS", rustflags_envvar)
        .env("CC", cc_path)
        .env("CFLAGS_riscv32im_risc0_zkvm_elf", c_flags)
        .args(args);

    println!{"~~~~~~ {:?}", cmd};

    cmd
}

pub fn sanitized_cmd(tool: &str) -> Command {
    let mut cmd = Command::new(tool);
    for (key, _val) in env::vars().filter(|x| x.0.starts_with("CARGO")) {
        cmd.env_remove(key);
    }
    cmd.env_remove("RUSTUP_TOOLCHAIN");
    cmd
}

/// Get the path used by cargo-risczero that stores downloaded toolchains
pub fn risc0_data() -> Result<PathBuf> {
    let dir = if let Ok(dir) = std::env::var("RISC0_DATA_DIR") {
        dir.into()
    } else if let Some(root) = dirs::data_dir() {
        root.join("cargo-risczero")
    } else if let Some(home) = dirs::home_dir() {
        home.join(".cargo-risczero")
    } else {
        anyhow::bail!("Could not determine cargo-risczero data dir. Set RISC0_DATA_DIR env var.");
    };

    Ok(dir)
}
