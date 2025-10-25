use anyhow::Result;
use libbpf_rs::skel::{OpenSkel, Skel, SkelBuilder};
use std::mem::MaybeUninit;

mod bpfprog;
use bpfprog::Bpfprog;

mod pid_iter {
    include!(concat!(env!("OUT_DIR"), "/pid_iter.skel.rs"));
}
use pid_iter::PidIterSkelBuilder;

fn load_pid_iter(iter_link: &mut Option<libbpf_rs::Link>) -> Result<()> {

    let prev_print_fn = unsafe {
        libbpf_sys::libbpf_set_print(None)
    };

    let result = (|| -> Result<()> {
        let skel_builder = PidIterSkelBuilder::default();
        let mut open_object = MaybeUninit::uninit();
        let open_skel = skel_builder.open(&mut open_object)?;
        let mut skel = open_skel.load()?;
        skel.attach()?;
        *iter_link = skel.links.bpf_iter;
        Ok(())
    })();

    unsafe {
        libbpf_sys::libbpf_set_print(prev_print_fn); 
    }
    
    result
}

fn main() -> Result<()> {

    let mut iter_link: Option<libbpf_rs::Link> = None;
    match load_pid_iter(&mut iter_link) {
        Ok(()) => println!("Successfully loaded pid_iter BPF program"),
        Err(e) => println!("Failed to load pid_iter BPF program: {}, continuing without process information", e),
    }


    Ok(())
}
