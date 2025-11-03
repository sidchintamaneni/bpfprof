//use std::env;
//use std::ffi::OsStr;
//use std::path::PathBuf;
//use std::process::Command;
//
//use libbpf_cargo::SkeletonBuilder;
//
//const SRC: &str = "src/bpf/pid_iter.bpf.c";

fn main() {
    //let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR must be set in build script"))
    //                        .join("pid_iter.skel.rs");
    //let vmlinux_path = PathBuf::from("src/bpf/vmlinux.h");

    //if !vmlinux_path.exists() {
    //    let output = Command::new("bpftool")
    //        .args(&["btf", "dump", "file", "/sys/kernel/btf/vmlinux", "format", "c"])
    //        .output()
    //        .expect("Failed to execute bpftool");

    //    if !output.status.success() {
    //        panic!(
    //            "Failed to generate vmlinux.h: {}",
    //            String::from_utf8_lossy(&output.stderr)
    //        );
    //    }

    //    std::fs::write(&vmlinux_path, output.stdout)
    //            .expect("Failed to write vmlinux.h");
    //}

    //SkeletonBuilder::new()
    //    .source(SRC)
    //    .clang_args([OsStr::new("-I")])
    //    .build_and_generate(&out)
    //    .unwrap();

}
