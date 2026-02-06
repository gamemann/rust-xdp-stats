use anyhow::Result;
use aya::programs::{ProgramError, Xdp, XdpFlags, xdp::XdpLinkId};

pub fn attach(prog: &mut Xdp, iface: &str, flags: XdpFlags) -> Result<XdpLinkId, ProgramError> {
    prog.attach(iface, flags)
}
