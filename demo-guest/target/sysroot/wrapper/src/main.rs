use std::borrow::Cow;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{self, Path, PathBuf};
use std::process::exit;

mod args;
mod toolchain_flags;
mod clang_parser;

use toolchain_flags::Args;
use clang_parser::parse_clang_cmd;

fn usage() -> ! {
    eprintln!("This multi-call binary should be used as one of the following names:
- hyperlight-config
- clang
");
    exit(1);
}

fn hyperlight_config(args: &Args, mut cmd: env::ArgsOs) {
    while let Some(flag) = cmd.next() {
        if flag == "--cflags" {
            println!("{}", toolchain_flags::cflags(&args, false).joined().to_str().unwrap());
        } else if flag == "--ldflags" {
            println!("{}", toolchain_flags::ldflags(&args).joined().to_str().unwrap());
        } else if flag == "--libs" {
            println!("{}", toolchain_flags::libs(&args).joined().to_str().unwrap());
        } else {
            eprintln!("Unknown flag `{}`", flag.display());
            exit(1);
        }
    }
}

/// Since path entries may not exist (especially on Windows),
/// use the original if there are any errors.
fn normalize(x: &Path) -> Cow<'_, Path> {
    if let Ok(y) = fs::canonicalize(x) {
        Cow::Owned(y)
    } else {
        Cow::Borrowed(x)
    }
}

fn find_next(args: &Args, tool_name: &str) -> PathBuf {
    let var_name = format!("HYPERLIGHT_TOOLCHAIN_{}", tool_name.to_lowercase().replace("-", "_"));
    if let Some(path) = env::var_os(var_name) {
        return path.into();
    }
    let wrapper_dir = normalize(&args.wrapper_dir);
    let path = env::var_os("PATH").expect("$PATH should exist");
    for path in env::split_paths(&path) {
        let abs_path = normalize(&path);
        if abs_path == wrapper_dir {
            continue;
        }
        let base_path = path.join(tool_name);
        if base_path.exists() {
            return base_path;
        }
        let exe_path = base_path.with_extension("exe");
        if exe_path.exists() {
            return exe_path;
        }
    }
    panic!("Could not find base system implementation of {}", tool_name);
}

fn clang(args: &Args, tool_name: &str, cmd: env::ArgsOs) {
    let cmd: Vec<_> = cmd.collect();
    let info = parse_clang_cmd(cmd.iter().map(<OsString as AsRef<OsStr>>::as_ref));
    let mut proc = std::process::Command::new(find_next(args, tool_name));
    proc.args(toolchain_flags::cflags(&args, false).0);
    if info.will_link() {
        proc.args(toolchain_flags::ldflags(&args).0);
    }
    proc.args(cmd);
    // TODO: should we auto-add libs? It's probably more flexible to not
    proc.status().expect("Failed to run system clang");
}

fn main() {
    let mut cmd = env::args_os();
    let Some(bin) = cmd.next() else { usage(); };
    let mut bin = PathBuf::from(bin);
    if bin.extension() == Some("exe".as_ref()) {
        bin.set_extension("");
    }
    let bin_name = bin.file_name().unwrap().to_str().unwrap();

    let sysroot_base = bin.parent().unwrap().parent().unwrap();
    // use absolute instead of fs::canonicalize because Clang does not
    // properly support \\?\ paths in include paths.
    let sysroot_base = path::absolute(sysroot_base).unwrap();
    let args = args::args(&sysroot_base);

    if bin_name == "hyperlight-config" {
        hyperlight_config(&args, cmd);
    } else if bin_name == "clang" || bin_name == format!("{0}-clang", args.target) {
        clang(&args, &bin_name, cmd);
    } else {
        usage();
    }
}
