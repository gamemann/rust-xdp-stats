use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct CliOpts {
    #[arg(
        short = 'i',
        long = "iface",
        default_value = "eth0",
        help = "The interface(s) to attach the XDP program to"
    )]
    pub iface: String,

    #[arg(
        short = 'd',
        long = "duration",
        default_value_t = 0,
        help = "The amount of time in seconds to run the program for (0 = infinite)."
    )]
    pub duration: u64,

    #[arg(
        short = 'a',
        long = "afxdp",
        default_value_t = false,
        help = "Forwards and processes packets in AF_XDP sockets instead of the raw XDP program."
    )]
    pub afxdp: bool,

    #[arg(
        short = 'n',
        long = "num-socks",
        default_value_t = 0,
        help = "The amount of AF_XDP sockets to create when running in AF_XDP mode."
    )]
    pub afxdp_num_socks: u32,

    #[arg(
        short = 's',
        long = "skb",
        default_value_t = false,
        help = "If set, attaches the XDP program using SKB mode (slower) instead of DRV."
    )]
    pub skb: bool,

    #[arg(
        short = 'o',
        long = "offload",
        default_value_t = false,
        help = "If set, attaches the XDP program using offload mode instead of DRV."
    )]
    pub offload: bool,

    #[arg(
        short = 'z',
        long = "replace",
        default_value_t = false,
        help = "If set, will attempt to replace the XDP program if it is already attached to the interface(s)."
    )]
    pub replace: bool,
}

impl CliOpts {
    pub fn get_ifaces(&self) -> Vec<String> {
        self.iface
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    }
}
