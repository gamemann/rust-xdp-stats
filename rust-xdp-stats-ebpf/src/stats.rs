use rust_xdp_stats_common::{StatType, Stats};

#[inline(always)]
pub fn inc_stats(stats: &mut Stats, stats_type: StatType, length: u64) {
    match stats_type {
        StatType::PASS => {
            stats.pass_pkt += 1;
            stats.pass_byt += length;
        }
        StatType::DROP => {
            stats.drop_pkt += 1;
            stats.drop_byt += length;
        }
        StatType::BAD => {
            stats.bad_pkt += 1;
            stats.bad_byt += length;
        }
        StatType::MATCH => {
            stats.match_pkt += 1;
            stats.match_byt += length;
        }
        StatType::FWD => {
            stats.fwd_pkt += 1;
            stats.fwd_byt += length;
        }
    }
}
