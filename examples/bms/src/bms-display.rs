// examples/bms/src/bms-display.rs

/*
 * Copyright (C) 2015-2023 IoT.bzh
 * SPDX-License-Identifier: MIT
 */

extern crate serde;
extern crate sockcan;

include!("./__bms-dbcgen.rs");
use crate::DbcSimple::CanMsgPool;

use clap::Parser;
use log::{info, warn};
use sockcan::prelude::*;

/// Read CAN messages and decode them with the generated DBC parser (BCM mode).
///
/// Examples:
///   bms-display                           # defaults: --iface vcan0 --rate 500 --watchdog 500
///   bms-display --iface can0              # pick real interface
///   bms-display -i vcan0 -r 200 -w 1000   # custom timers (ms)
#[derive(Debug, Parser)]
#[command(name = "bms-display", version, about, author)]
struct Args {
    /// CAN interface name
    #[arg(short = 'i', long = "iface", default_value = "vcan0")]
    iface: String,

    /// BCM receive timer period in milliseconds (SET_TIMER)
    #[arg(short = 'r', long = "rate", default_value_t = 500, value_parser = clap::value_parser!(u64).range(1..=60_000))]
    rate_ms: u64,

    /// BCM watchdog timeout in milliseconds (START_TIMER)
    #[arg(short = 'w', long = "watchdog", default_value_t = 500, value_parser = clap::value_parser!(u64).range(1..=300_000))]
    watchdog_ms: u64,

    /// Increase verbosity (can be repeated: -v, -vv)
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    verbose: u8,
}

fn init_logging(verbosity: u8) {
    // map -v levels to env_logger filters
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let env = env_logger::Env::default().default_filter_or(level);
    let _ = env_logger::Builder::from_env(env).format_timestamp_millis().try_init();
}

fn register_pool_filters(
    sock: &SockCanHandle,
    pool: &CanMsgPool,
    rate_ms: u64,
    watchdog_ms: u64,
) -> Result<(), CanError> {
    for &canid in pool.get_ids().iter() {
        SockBcmCmd::new(
            CanBcmOpCode::RxSetup,
            CanBcmFlag::RX_FILTER_ID
                | CanBcmFlag::SET_TIMER
                | CanBcmFlag::START_TIMER
                | CanBcmFlag::RX_ANNOUNCE_RESUME,
            canid,
        )
        .set_timers(rate_ms, watchdog_ms)
        .apply(sock)?;
        info!("Subscribed canid=0x{canid:03X} rate={}ms watchdog={}ms", rate_ms, watchdog_ms);
    }
    Ok(())
}

fn main() -> Result<(), CanError> {
    let args = Args::parse();
    init_logging(args.verbose);

    info!("Opening BCM socket on iface {}", args.iface);
    let sock = SockCanHandle::open_bcm(args.iface.as_str(), CanTimeStamp::CLASSIC)?;

    let pool = CanMsgPool::new("dbc-demo");
    if pool.get_ids().is_empty() {
        warn!("DBC pool returned no IDs — nothing to subscribe.");
    }

    register_pool_filters(&sock, &pool, args.rate_ms, args.watchdog_ms)?;

    let mut count: u64 = 0;
    loop {
        count = count.saturating_add(1);

        // Read a BCM message (only filtered CAN IDs should arrive)
        let bcm_msg = sock.get_bcm_frame();

        // Prepare message for DBC parsing
        let msg_data = CanMsgData {
            canid: bcm_msg.get_id()?,
            stamp: bcm_msg.get_stamp(),
            opcode: bcm_msg.get_opcode(),
            len: bcm_msg.get_len()?,
            data: bcm_msg.get_data()?,
        };

        // Feed into the parser pool
        let msg = pool.update(&msg_data)?;
        println!(
            "\n({count}) => canid:0x{:03X} opcode:{:?} stamp:{}",
            msg_data.canid, msg_data.opcode, msg_data.stamp
        );

        for sig_ref in msg.get_signals() {
            let signal = sig_ref.borrow();
            let age_ms = if signal.get_stamp() > 0 {
                (msg_data.stamp.saturating_sub(signal.get_stamp())) / 1000
            } else {
                0
            };

            let json = if cfg!(feature = "serde") {
                signal.to_json()
            } else {
                "serde-disabled".to_owned()
            };

            println!(
                "  -- {:<20} value:{:<12?} status:{:<8?} age:{:>6} ms\n     json:{}",
                signal.get_name(),
                signal.get_value(),
                signal.get_status(),
                age_ms,
                json
            );
        }
    }
}
