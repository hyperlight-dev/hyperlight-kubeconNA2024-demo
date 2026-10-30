use std::ffi::OsStr;

#[derive(Copy, Clone)]
pub(crate) struct ClangCmdInfo {
    pub(crate) has_non_flag: bool,
    pub(crate) has_non_link_flag: bool,
}
impl ClangCmdInfo {
    pub(crate) fn will_link(self) -> bool {
        !self.has_non_link_flag && self.has_non_flag
    }
}

pub(crate) fn parse_clang_cmd<'a>(cmd: impl Iterator<Item = &'a OsStr>) -> ClangCmdInfo {
    // We can add cflags unconditionally, but to know whether or not
    // to add ldflags, we need to know whether this compiler driver
    // invocation is expected to link or not.
    //
    // We look through the arguments to produce two pieces of state:
    // - `found_non_link_flag`: If true, we found some flag (like -c)
    //   which means that the compiler driver will not invoke the
    //   linker, so we should not include the linker
    //   arguments. Currently, we detect:
    //   + `-c`, `-S`, `-E`, `-M`, `-MM`
    //   + `-x <lang>-header`
    // - `found_non_flag`: If true, there was at least one non-flag
    //   argument, so the driver will actually be compiling
    //   something. This is important to catch someone running
    //   e.g. `clang -v` and avoid inserting the linker arguments,
    //   which would trigger a great number of warnings.
    let mut flags_valid: bool = true;
    let mut found_non_link_flag: bool = false;
    let mut found_non_flag: bool = false;
    let mut iter = cmd.map(OsStr::to_str).map(Option::unwrap);
    while let Some(arg) = iter.next() {
        if !flags_valid || &arg[0..1] != "-" {
            found_non_flag = true;
        }
        if arg == "--" {
            flags_valid = false;
        }
        if flags_valid && &arg[0..1] == "-" {
            let flag = &arg[1..];
            if flag == "c" || flag == "S" || flag == "E" || flag == "M" || flag == "MM" {
                found_non_link_flag = true;
            }
            if &flag[0..1] == "x" {
                let lang = if flag.len() > 1 {
                    Some(&flag[1..])
                } else {
                    iter.next()
                };
                if let Some(lang) = lang
                    && lang.ends_with("-header")
                {
                    found_non_link_flag = true;
                }
            }
        }
    }
    ClangCmdInfo {
        has_non_link_flag: found_non_link_flag,
        has_non_flag: found_non_flag,
    }
}
