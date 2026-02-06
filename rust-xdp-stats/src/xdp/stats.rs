use anyhow::Result;

use aya::maps::{PerCpuArray, PerCpuValues};

use rust_xdp_stats_common::Stats;

use aya::Ebpf;
use aya::util::nr_cpus;

pub fn init_stats(bpf: &mut Ebpf) -> Result<()> {
    let nr_cpus = nr_cpus().map_err(|(_, error)| error)?;

    // We need to insert an empty structure into the stats map.
    let stats_key: u32 = 0;

    let stat_val: Stats = Stats::default();

    let stats_vals = PerCpuValues::try_from(vec![stat_val; nr_cpus])?;

    // We need to retrieve the map.
    let mut stats_map = PerCpuArray::try_from(bpf.map_mut("MAP_STATS").unwrap())?;

    stats_map.set(stats_key, stats_vals, 0)?;

    Ok(())
}

pub fn get_stats(bpf: &mut Ebpf) -> Result<Stats> {
    let stats_key: u32 = 0;

    let stats_map = PerCpuArray::try_from(bpf.map_mut("MAP_STATS").unwrap())?;

    let mut stats_ret: Stats = Stats::default();

    let stats: PerCpuValues<Stats> = stats_map.get(&stats_key, 0)?;

    for stat in stats.iter() {
        stats_ret.pass_pkt += stat.pass_pkt;
        stats_ret.pass_byt += stat.pass_byt;
        stats_ret.drop_pkt += stat.drop_pkt;
        stats_ret.drop_byt += stat.drop_byt;
        stats_ret.bad_pkt += stat.bad_pkt;
        stats_ret.bad_byt += stat.bad_byt;
        stats_ret.match_pkt += stat.match_pkt;
        stats_ret.match_byt += stat.match_byt;
        stats_ret.fwd_pkt += stat.fwd_pkt;
        stats_ret.fwd_byt += stat.fwd_byt;
    }

    Ok(stats_ret)
}
