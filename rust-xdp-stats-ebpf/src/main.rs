#![no_std]
#![no_main]

mod stats;
mod utils;

use stats::inc_stats;
use utils::ptr_at;

use rust_xdp_stats_common::{StatType, Stats, TARGET_PORT};

use aya_ebpf::{
    bindings::xdp_action,
    macros::{map, xdp},
    maps::PerCpuArray,
    programs::XdpContext,
};

use aya_log_ebpf::info;

use xdp_action::{XDP_ABORTED, XDP_DROP, XDP_PASS};

use network_types::{
    eth::{EthHdr, EtherType},
    ip::{IpProto, Ipv4Hdr},
    udp::UdpHdr,
};

#[map]
static MAP_STATS: PerCpuArray<Stats> = PerCpuArray::with_max_entries(1, 0);

#[inline(always)]
fn exit_prog(pkt_len: u32, stats: &mut Stats, stats_type: StatType, ret: xdp_action::Type) -> u32 {
    inc_stats(stats, stats_type, pkt_len as u64);

    ret
}

#[xdp]
pub fn rust_xdp_stats(ctx: XdpContext) -> u32 {
    // We can retrieve the total packet length by subtracting data from data_end.
    let pkt_len = (ctx.data_end() - ctx.data()) as u32;

    // We need to retrieve the stats map.
    let stats_key: u32 = 0;

    let stats = match MAP_STATS.get_ptr_mut(stats_key) {
        Some(stats) => unsafe { &mut *stats },
        None => {
            info!(&ctx, "failed to retrieve stats map.");

            return XDP_DROP;
        }
    };

    // We need to initialize the ethernet header and check.
    let eth: *const EthHdr = match unsafe { ptr_at(&ctx, 0) } {
        Ok(eth) => eth,
        Err(_) => return exit_prog(pkt_len, stats, StatType::BAD, XDP_ABORTED),
    };

    // We need to pass packets to the Linux network stack if they aren't an IPv4 packet.
    match unsafe { (*eth).ether_type() } {
        Ok(EtherType::Ipv4) => {}
        _ => return exit_prog(pkt_len, stats, StatType::PASS, XDP_PASS),
    }

    // Initialize and check IPv4 header.
    let iph: *const Ipv4Hdr = match unsafe { ptr_at(&ctx, EthHdr::LEN) } {
        Ok(iph) => iph,
        Err(_) => return exit_prog(pkt_len, stats, StatType::BAD, XDP_ABORTED),
    };

    // If the protocol isn't UDP, pass to network stack.
    match unsafe { (*iph).proto } {
        IpProto::Udp => {}
        _ => return exit_prog(pkt_len, stats, StatType::PASS, XDP_PASS),
    }

    // Retrieve IP header length.
    // NOTE: Dynamically retrieving the IP header length and doing a check results in a bad packet. This is likely due to the verifier, but I don't have this issue in C. IPv4 header should be 20 bytes anyways though ¯\_(ツ)_/¯
    //let ip_len = (unsafe { (*iph).ihl() } as usize) * 4;
    let ip_len = Ipv4Hdr::LEN;

    // We need to retrieve the UDP header.
    let udph: *const UdpHdr = match unsafe { ptr_at(&ctx, EthHdr::LEN + ip_len) } {
        Ok(udph) => udph,
        Err(_) => return exit_prog(pkt_len, stats, StatType::BAD, XDP_ABORTED),
    };

    // Retrieve destination port and check.
    let dst_port = unsafe { (*udph).dst_port() };

    // If the target port isn't right, pass to network stack.
    if dst_port != TARGET_PORT {
        return exit_prog(pkt_len, stats, StatType::PASS, XDP_PASS);
    }

    exit_prog(pkt_len, stats, StatType::MATCH, XDP_DROP)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[unsafe(link_section = "license")]
#[unsafe(no_mangle)]
static LICENSE: [u8; 13] = *b"Dual MIT/GPL\0";
