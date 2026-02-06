pub mod cli;
pub mod utils;
pub mod xdp;

use cli::CliOpts;
use utils::raise_rlimit;
use xdp::{attach, init_stats, load_bpf, load_xdp};

use anyhow::Result;
use aya::programs::XdpFlags;
#[rustfmt::skip]
use log::{debug, warn, info};
use std::time::Duration;
use tokio::{select, signal};

use tokio::time;

use std::io::{self, Write};

use env_logger::Env;

use clap::Parser;

use aya_log::EbpfLogger;

#[tokio::main]
async fn main() -> Result<()> {
    // Let's parse our CLI options first and extract the input.
    let opts = match CliOpts::try_parse() {
        Ok(opts) => opts,
        Err(e) => {
            eprintln!("{e}");

            std::process::exit(1);
        }
    };

    let CliOpts {
        iface: _,
        duration,
        afxdp: _,
        afxdp_num_socks: _,
        skb,
        offload,
        replace,
    } = opts;

    // We need to retrieve the list of interfaces to attach to.
    let mut ifaces = opts.get_ifaces();

    // If we don't have any interfaces, we can try eth0, but warn.
    if ifaces.is_empty() {
        warn!("no interfaces specified, attempting to use 'eth0'");

        ifaces.push("eth0".to_string());
    }

    // We need to set up our log environment now.
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // We need to raise the RLimit for older kernels.
    match raise_rlimit() {
        Err(e) => warn!("Failed to raise rlimit: {e}"),
        Ok(_) => debug!("Successfully raised rlimit"),
    };

    // We need to load our eBPF program before attaching it to the interface(s).
    let mut bpf_prog = load_bpf().expect("Failed to load BPF object file");

    match EbpfLogger::init(&mut bpf_prog) {
        Err(_) => {} // We don't care for logging since we don't log directly in BPF by default (only when debugging).
        Ok(logger) => {
            let mut logger =
                tokio::io::unix::AsyncFd::with_interest(logger, tokio::io::Interest::READABLE)?;
            tokio::task::spawn(async move {
                loop {
                    let mut guard = logger.readable_mut().await.unwrap();
                    guard.get_inner_mut().flush();
                    guard.clear_ready();
                }
            });
        }
    }

    // Retrieve XDP program.
    let xdp_prog = load_xdp(&mut bpf_prog, "rust_xdp_stats").expect("failed to load XDP program");

    // Before attaching the XDP program, let's compile the attach flags from input.
    let mut attach_flags = match (skb, offload) {
        (true, false) => XdpFlags::SKB_MODE,
        (false, true) => XdpFlags::HW_MODE,
        _ => XdpFlags::default(),
    };

    // Apply the replace flag if wanted.
    // This would be ideal by default, but
    // For some reason this causes a panic with:
    // called `Option::unwrap()` on a `None` value
    if replace {
        attach_flags |= XdpFlags::REPLACE;
    }

    // Now attempt to load XDP programs on interfaces specified.
    let mut is_attached = false;

    for iface in ifaces {
        match attach(xdp_prog, iface.as_str(), attach_flags) {
            Ok(_) => {
                if !is_attached {
                    is_attached = true;
                }

                info!("Attached XDP program to interface {iface}...");
            }
            Err(e) => warn!("Failed to attach XDP program to interface '{iface}': {e}"),
        }
    }

    // If we aren't attached, exit.
    if !is_attached {
        return Err(anyhow::anyhow!(
            "Failed to attach XDP program to any interface"
        ));
    }

    // Attempt to insert first stats entry value.
    match init_stats(&mut bpf_prog) {
        Ok(_) => debug!("Successfully inserted first stats entry value"),
        Err(e) => warn!("Failed to insert first stats entry value: {e}"),
    }

    info!("Rust XDP Stats loaded! Please use CTRL + C to exit...");

    // We need to calculate our interval (one second).
    let mut interval = time::interval(Duration::from_secs(1));

    let mut elapsed = 0;

    loop {
        select! {
            _ = interval.tick() => {
                // Retrieve stats.
                match xdp::stats::get_stats(&mut bpf_prog) {
                    Ok(stats) => {
                        print!("\r\x1b[1;32mPassed:\x1b[0m {}/{}  |  ", stats.pass_pkt, stats.pass_byt);
                        print!("\x1b[1;31mDropped:\x1b[0m {}/{}  |  ", stats.drop_pkt, stats.drop_byt);
                        print!("\x1b[1;33mBad:\x1b[0m {}/{}  |  ", stats.bad_pkt, stats.bad_byt);
                        print!("\x1b[1;34mMatched:\x1b[0m {}/{}  |  ", stats.match_pkt, stats.match_byt);
                        print!("\x1b[1;35mForwarded:\x1b[0m {}/{}", stats.fwd_pkt, stats.fwd_byt);

                        io::stdout().flush()?;
                    },
                    Err(e) => warn!("Failed to retrieve stats: {e}"),
                }

                // Increment our elapsed time and check if we should exit.
                if duration > 0 {
                    elapsed +=1;

                    if elapsed >= duration {
                        info!("Time exceeded ({} seconds), exiting...", duration);

                        break;
                    }
                }
            }

            _ = signal::ctrl_c() => {
                info!("Found CTRL + C signal, exiting...");

                break;
            }
        }
    }

    println!();
    info!("Rust XDP Stats cleaned up and exiting...");

    Ok(())
}
