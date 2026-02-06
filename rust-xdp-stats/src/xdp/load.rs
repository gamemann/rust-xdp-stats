use rust_xdp_stats_common::PATH_ELF_FILE;

use anyhow::Result;
use aya::{Ebpf, programs::Xdp};

pub fn load_bpf() -> Result<Ebpf> {
    // We need to build our ELF path to load with eBPF.
    let elf_path = format!("{}/{}", env!("OUT_DIR"), PATH_ELF_FILE);

    // Attempt to load our eBPF program.
    Ok(Ebpf::load_file(elf_path)?)
}

pub fn load_xdp<'a>(bpf: &'a mut Ebpf, sec_name: &str) -> Result<&'a mut Xdp> {
    // Now attempt to load our XDP program.
    let prog: &mut Xdp = bpf
        .program_mut(sec_name)
        .ok_or_else(|| anyhow::anyhow!("Section not found"))?
        .try_into()?;

    prog.load()?;

    Ok(prog)
}
