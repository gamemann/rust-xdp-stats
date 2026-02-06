#[derive(Debug, Clone)]
pub enum StatType {
    PASS,
    DROP,
    BAD,
    MATCH,
    FWD,
}

#[repr(C)]
#[derive(Copy, Default, Debug, Clone)]
pub struct Stats {
    pub pass_pkt: u64,
    pub pass_byt: u64,

    pub drop_pkt: u64,
    pub drop_byt: u64,

    pub bad_pkt: u64,
    pub bad_byt: u64,

    pub match_pkt: u64,
    pub match_byt: u64,

    pub fwd_pkt: u64,
    pub fwd_byt: u64,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for Stats {}
