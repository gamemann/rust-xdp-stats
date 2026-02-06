#[derive(Debug, Clone)]
pub struct AfXdpCtx {
    pub num_socks: u32,
    pub skb: bool,
    pub zerocopy: bool,
    pub copy: bool,
}
