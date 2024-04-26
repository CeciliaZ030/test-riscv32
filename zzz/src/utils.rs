use cargo_metadata::{Message, Metadata, Target};
use chrono::Local;
use regex::Regex;
use std::io::BufRead;
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

use crate::Executor;

#[test]
fn testt() {
    let meta = super::parse_metadata("../cunt");
    let tests = meta.tests();
    let bins = meta.bins();
    let libs = meta.libs();
    [tests, bins, libs].iter().for_each(|ps| {
        let names = ps.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
        println!("{:?}\n", names);
    });

    let builder = GuestBuilder::new(meta, "riscv32im-succinct-zkvm-elf", "succinct")
        .rust_flags(&[
            "passes=loweratomic",
            "link-arg=-Ttext=0x00200800",
            "panic=abort",
        ])
        .custom_args(&["--ignore-rust-version"]);
    let mut cmd = builder.build_command("release", None);
    println!("\n{:?}", cmd);

    let res = cmd.status().expect("Failed to run cargo command.");
    assert!(res.success());
}

pub trait GuestMetadata {
    // /// Kind of target ("bin", "example", "test", "bench", "lib", "custom-build")
    fn tests(&self) -> Vec<&Target>;
    fn bins(&self) -> Vec<&Target>;
    fn examples(&self) -> Vec<&Target>;
    fn benchs(&self) -> Vec<&Target>;
    fn libs(&self) -> Vec<&Target>;
    fn build_scripts(&self) -> Vec<&Target>;
}

impl GuestMetadata for Metadata {
    fn tests(&self) -> Vec<&Target> {
        self.packages.iter().fold(Vec::new(), |mut packages, p| {
            packages.extend(p.targets.iter().filter(|t| t.test));
            packages
        })
    }

    fn bins(&self) -> Vec<&Target> {
        self.packages.iter().fold(Vec::new(), |mut packages, p| {
            packages.extend(
                p.targets
                    .iter()
                    .filter(|t| t.kind.iter().any(|k| k == "bin")),
            );
            packages
        })
    }

    fn examples(&self) -> Vec<&Target> {
        self.packages.iter().fold(Vec::new(), |mut packages, p| {
            packages.extend(
                p.targets
                    .iter()
                    .filter(|t| t.kind.iter().any(|k| k == "example")),
            );
            packages
        })
    }

    fn benchs(&self) -> Vec<&Target> {
        self.packages.iter().fold(Vec::new(), |mut packages, p| {
            packages.extend(
                p.targets
                    .iter()
                    .filter(|t| t.kind.iter().any(|k| k == "bench")),
            );
            packages
        })
    }

    fn libs(&self) -> Vec<&Target> {
        self.packages.iter().fold(Vec::new(), |mut packages, p| {
            packages.extend(
                p.targets
                    .iter()
                    .filter(|t| t.kind.iter().any(|k| k == "lib")),
            );
            packages
        })
    }

    fn build_scripts(&self) -> Vec<&Target> {
        self.packages.iter().fold(Vec::new(), |mut packages, p| {
            packages.extend(
                p.targets
                    .iter()
                    .filter(|t| t.kind.iter().any(|k| k == "custom-build")),
            );
            packages
        })
    }
}

#[derive(Clone)]
pub struct GuestBuilder {
    pub meta: Metadata,

    pub target: String,

    pub sanitized_env: Vec<String>,

    pub cargo: PathBuf,

    // rustc compiler specific to toolchain
    pub rustc_compiler: PathBuf,
    // -C flags
    pub rust_flags: Option<Vec<String>>,
    // -Z flags
    pub z_flags: Option<Vec<String>>,
    // riscv32im gcc
    pub cc_compiler: Option<PathBuf>,
    // gcc flag
    pub c_flags: Option<Vec<String>>,

    pub custom_args: Vec<String>,

    custom_env: HashMap<String, String>,
}

impl GuestBuilder {
    pub fn new(meta: &Metadata, target: &str, toolchain: &str) -> Self {
        let tools = ["cargo", "rustc"]
            .into_iter()
            .map(|tool| {
                let out = sanitized_cmd("rustup")
                    .args([format!("+{toolchain}").as_str(), "which", tool])
                    .output()
                    .expect("rustup failed to find {toolchain} toolchain")
                    .stdout;
                let out = String::from_utf8(out).unwrap();
                let out = out.trim();
                println!("Using rustc: {out}");
                PathBuf::from(out)
            })
            .collect::<Vec<_>>();
        Self {
            meta: meta.clone(),
            target: target.to_string(),
            sanitized_env: Vec::new(),
            cargo: tools[0].clone(),
            rustc_compiler: tools[1].clone(),
            rust_flags: None,
            z_flags: None,
            cc_compiler: None,
            c_flags: None,
            custom_args: Vec::new(),
            custom_env: HashMap::new(),
        }
    }

    fn sanitized_env(mut self, env_vars: &[&str]) -> Self {
        self.sanitized_env = to_strings(env_vars);
        self
    }

    pub fn rust_flags(mut self, flags: &[&str]) -> Self {
        self.rust_flags = Some(to_strings(flags));
        self
    }

    pub fn z_flags(mut self, flags: &[&str]) -> Self {
        self.z_flags = Some(to_strings(flags));
        self
    }

    pub fn cc_compiler(mut self, compiler: PathBuf) -> Self {
        self.cc_compiler = Some(compiler);
        self
    }

    pub fn c_flags(mut self, flags: &[&str]) -> Self {
        self.c_flags = Some(to_strings(flags));
        self
    }

    pub fn custom_args(mut self, args: &[&str]) -> Self {
        self.custom_args = to_strings(args);
        self
    }

    pub fn custom_env(mut self, env: HashMap<String, String>) -> Self {
        self.custom_env = env;
        self
    }

    pub fn extend_custom(&self, cmd: &mut Command, args: &mut Vec<String>) {
        args.extend(self.custom_args.clone());
        for (key, val) in self.custom_env.iter() {
            cmd.env(key, val);
        }
    }

    pub fn sanitize(&self, cmd: &mut Command, filter_cargo: bool) {
        if filter_cargo {
            for (key, _val) in env::vars().filter(|x| x.0.starts_with("CARGO")) {
                cmd.env_remove(key);
            }
        }
        self.sanitized_env.iter().for_each(|e| {
            cmd.env_remove(e);
        });
    }

    pub fn build_command(&self, profile: &str, bins: Option<Vec<&str>>) -> Executor {
        let args = vec!["build".to_string()];
        let cmd = self.inner_command(args, profile, bins.clone().map(|b| to_strings(&b)));
        // target/
        // ├── debug/
        //    ├── deps/
        //    │   |── main-<hasha>   --> this is the output
        //    │   |── main-<hashb>
        //    │   |── bin1-<hashc>   --> this is the output
        //    │   |── bin1-<hashd>
        //    │   └── bin2-<hashe>   --> this is the output
        //    ├── build/
        //    ├── main               --> this is the output (same)
        //    ├── bin1               --> this is the output (same)
        //    └── bin2               --> this is the output (same)
        let target_path: PathBuf = self
            .meta
            .target_directory
            .join(self.target.clone())
            .join(profile.to_string())
            .into();
        let artifacts = self
            .meta
            .bins()
            .iter()
            .filter(|t| {
                if let Some(bins) = &bins {
                    bins.contains(&t.name.as_str())
                } else {
                    true
                }
            })
            .map(|t| target_path.join(t.name.clone()))
            .collect::<Vec<_>>();

        Executor {
            cmd,
            artifacts,
            test: false,
        }
    }

    pub fn test_command(&self, profile: &str, bins: Option<Vec<&str>>) -> Executor {
        let args = vec!["test".to_string(), "--no-run".to_string()];
        let cmd = self.inner_command(args, profile, bins.clone().map(|b| to_strings(&b)));
        // target/
        // ├── debug/
        //    ├── deps/
        //    │   |── main-<hasha>
        //    │   |── main-<hashb>    --> this is the test
        //    │   |── bin1-<hashc>
        //    │   |── bin1-<hashd>    --> this is the test
        //    │   |── bin2-<hashe>
        //    │   └── my-test-<hashe> --> this is the test
        //    ├── build/
        // Thus the test artifacts path are hypothetical because we don't know the hash yet
        let target_path: PathBuf = self
            .meta
            .target_directory
            .join(self.target.clone())
            .join(profile.to_string())
            .join("deps")
            .into();
        let artifacts = self
            .meta
            .tests()
            .iter()
            .filter(|t| {
                if let Some(bins) = &bins {
                    bins.contains(&t.name.as_str())
                } else {
                    true
                }
            })
            .map(|t| target_path.join(t.name.clone()))
            .collect::<Vec<_>>();

        Executor {
            cmd,
            artifacts,
            test: true,
        }
    }

    pub fn inner_command(
        &self,
        mut args: Vec<String>,
        profile: &str,
        bins: Option<Vec<String>>,
    ) -> Command {
        let GuestBuilder {
            meta,
            target,
            cargo,
            rustc_compiler,
            rust_flags,
            z_flags,
            cc_compiler,
            c_flags,
            ..
        } = self.clone();

        // Construct cargo args
        // `--{profile} {bin} --target {target} --locked -Z {z_flags}`
        if profile != "debug" {
            // error: unexpected argument '--debug' found; tip: `--debug` is the default
        }
        args.extend(vec![
            "--target".to_string(),
            target.clone(),
            "--locked".to_string(),
        ]);
        if let Some(bins) = bins {
            println!("{:?}", bins);
            println!("{:?}", format_flags("--bin", &bins));
            args.extend(format_flags("--bin", &bins));
        }
        if let Some(z_flags) = z_flags {
            args.extend(format_flags("-Z", &z_flags));
        }

        // Construct command from the toolchain-specific cargo
        let mut cmd = Command::new(cargo);
        // Clear unwanted env vars
        self.sanitize(&mut cmd, false);
        cmd.current_dir(meta.target_directory.parent().unwrap());

        // Set Rustc compiler path and flags
        cmd.env("RUSTC", rustc_compiler);
        if let Some(rust_flags) = rust_flags {
            cmd.env(
                "CARGO_ENCODED_RUSTFLAGS",
                format_flags("-C", &rust_flags).join("\x1f"),
            );
        }

        // Set C compiler path and flags
        if let Some(cc_compiler) = cc_compiler {
            cmd.env("CC", cc_compiler);
        }
        if let Some(c_flags) = c_flags {
            cmd.env(format!("CFLAGS_{}", self.target), c_flags.join(" "));
        }

        self.extend_custom(&mut cmd, &mut args);
        cmd.args(args);

        cmd
    }
}

fn to_strings(strs: &[&str]) -> Vec<String> {
    println!("{:?}", strs);
    let r = strs.iter().map(|s| s.to_string()).collect();
    println!("{:?}", r);
    r
}

pub fn format_flags(flag: &str, items: &Vec<String>) -> Vec<String> {
    let res = items.iter().fold(Vec::new(), |mut res, i| {
        res.extend([flag.to_owned(), i.to_owned()]);
        res
    });
    res
}

fn sanitized_cmd(tool: &str) -> Command {
    let mut cmd = Command::new(tool);
    for (key, _val) in env::vars().filter(|x| x.0.starts_with("CARGO")) {
        cmd.env_remove(key);
    }
    cmd.env_remove("RUSTUP_TOOLCHAIN");
    cmd
}
