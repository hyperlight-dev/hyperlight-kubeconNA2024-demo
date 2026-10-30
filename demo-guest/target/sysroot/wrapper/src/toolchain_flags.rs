// this file is used both from the cargo-hyperlight library (via
// `mod`) and from the wrapper and config scripts that are built for
// $HOST in a package specified in wrapper/_Cargo.toml. So, it can
// only use imports available in both those environments.

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

pub(crate) struct Args {
    pub(crate) includes_dir: PathBuf,
    pub(crate) c_libs_dir: PathBuf,
    #[allow(unused)] // used in the wrapper, but not the host
    pub(crate) wrapper_dir: PathBuf,
    pub(crate) target: String,
    pub(crate) with_guest_capi: bool,
}
impl Args {
    fn clang_target(&self) -> &'static OsStr {
        if self.target.starts_with("aarch64") {
            "--target=aarch64-unknown-linux-none".as_ref()
        } else {
            "--target=x86_64-unknown-linux-none".as_ref()
        }
    }
}

fn os(x: &(impl AsRef<OsStr> + ?Sized)) -> Cow<'_, OsStr> {
    x.as_ref().into()
}

pub struct Flags(pub(crate) Vec<Cow<'static, OsStr>>);
impl Flags {
    /// This will behave badly with spaces/special characters/etc,
    /// which is why it is called `joined` and not
    /// `joined_escaped`. That's OK for all the flags we use for now.
    pub fn joined(&self) -> OsString {
        let mut all = OsString::new();
        all.push(&self.0[0]);
        for flag in &self.0[1..] {
            all.push(os(" "));
            all.push(flag);
        }
        all
    }
}

pub(crate) fn cflags(args: &Args, bootstrap: bool) -> Flags {
    const COMMON_FLAGS: &[&str] = &[
        "-U__linux__",
        "-fPIC",
        // We don't support stack protectors at the moment, but Arch Linux clang
        // auto-enables them for -linux platforms, so explicitly disable them.
        "-fno-stack-protector",
        "-fstack-clash-protection",
        "-mstack-probe-size=4096",
        "-nostdlibinc",
        // Define HYPERLIGHT as we use this to conditionally enable/disable code
        // in the libc headers
        "-DHYPERLIGHT=1",
        "-D__HYPERLIGHT__=1",
    ];

    const X86_64_FLAGS: &[&str] = &["-mno-red-zone"];

    const AARCH64_FLAGS: &[&str] = &["-mstrict-align", "-march=armv8.1-a+fp+simd"];

    let mut flags: Vec<Cow<'_, OsStr>> = Vec::new();
    flags.push(args.clang_target().into());
    COMMON_FLAGS
        .iter()
        .chain(if args.target.starts_with("x86_64") {
            X86_64_FLAGS.iter()
        } else if args.target.starts_with("aarch64") {
            AARCH64_FLAGS.iter()
        } else {
            [].iter()
        })
        .for_each(|x| flags.push(os(x)));

    if !bootstrap {
        flags.push(os("-isystem"));
        flags.push(Cow::Owned(args.includes_dir.clone().into()));
    }
    Flags(flags)
}

pub(crate) fn ldflags(args: &Args) -> Flags {
    const COMMON_FLAGS: &[&str] = &[
        "-e",
        "entrypoint",
        "-nostdlib",
        "-pie",
        "-Wl,--no-dynamic-linker",
    ];

    let mut flags = Vec::new();
    flags.push(args.clang_target().into());
    COMMON_FLAGS.iter().for_each(|x| flags.push(os(x)));

    flags.push(os("-L"));
    flags.push(Cow::Owned(args.c_libs_dir.clone().into()));

    Flags(flags)
}

pub(crate) fn libs(args: &Args) -> Flags {
    let mut flags = Vec::new();
    if args.with_guest_capi {
        flags.push(os("-l"));
        flags.push(os("hyperlight_guest_capi"));
    }
    Flags(flags)
}
