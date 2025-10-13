/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 */

extern crate serde;
extern crate sockcan;

// include generated code and Rust module as declare in build.rs->DbcParser::new("DbcSimple")
include!("./__model3-dbcgen.rs");
use crate::DbcSimple::CanMsgPool;

use sockcan::prelude::*;
use std::env;
use std::str::FromStr;

/// read can messages and decode them with previously generated dbc parser
/// This example run in two steps
/// * At compilation time
///     * generates Rust DBC parser with build.`rs/DbcParser::::fromdbc()`
///     * include generated RUST dbc parser with socketcan api
///     * generate `can_display` binary
/// * At run time
///     * query dbc parser to get the list of supported canid
///     * subscribe to corresponding message with provided timers
///     * on message reception display decoded values
/// * usage:
///     * cargo build
///     * `can_display` vcan0 500 1000
fn main() -> Result<(), CanError> {
    let args: Vec<String> = env::args().collect();
    println!("syntax: {} [vcan0] [rate-ms] [watchdog-ms]", args[0]);

    let candev = if args.len() > 1 { args[1].as_str() } else { "vcan0" };
    let rate = if args.len() > 2 {
        u64::from_str(args[2].as_str()).expect("rate expect a valid integer")
    } else {
        500
    };
    let watchdog = if args.len() > 3 {
        u64::from_str(args[3].as_str()).expect("watch expect a valid integer")
    } else {
        500
    };

    // try to open candev (exit on Error)
    let sock = SockCanHandle::open_bcm(candev, CanTimeStamp::CLASSIC)?;

    // get canid list from dbc pool
    let pool = CanMsgPool::new("dbc-demo");

    // register dbc defined canid
    for canid in pool.get_ids() {
        SockBcmCmd::new(
            CanBcmOpCode::RxSetup,
            CanBcmFlag::RX_FILTER_ID
                | CanBcmFlag::SET_TIMER
                | CanBcmFlag::START_TIMER
                | CanBcmFlag::RX_ANNOUNCE_RESUME,
            *canid,
        )
        .set_timers(rate, watchdog)
        .apply(&sock)?;
    }

    // loop on message reception and decode messages
    let mut count = 0;
    loop {
        count += 1;

        // read a new bmc_msg (only filter canid should be received)
        let bmc_msg = sock.get_bcm_frame();

        // prepare message dbc for parsing
        let msg_data = CanMsgData {
            canid: bmc_msg.get_id()?,
            stamp: bmc_msg.get_stamp(),
            opcode: bmc_msg.get_opcode(),
            len: bmc_msg.get_len()?,
            data: bmc_msg.get_data()?,
        };

        // try to update message data within dbc parser pool
        let msg = pool.update(&msg_data)?;
        println!(
            "\n({}) => CanID:{} opcode:{:?} stamp:{}",
            count, msg_data.canid, msg_data.opcode, msg_data.stamp
        );

        // loop on message signal and display values.
        for sig_rfc in msg.get_signals() {
            let signal = sig_rfc.borrow();
            let stamp = signal.get_stamp();
            let mut sig_age_ms = 0;

            let json =
                if cfg!(feature = "serde") { signal.to_json() } else { "serde-disable".to_owned() };

            if stamp > 0 {
                sig_age_ms = (msg_data.stamp - signal.get_stamp()) / 1000;
            }
            println!(
                "  -- {:25} value:{:?} status:{:?} age:{} json:{}",
                signal.get_name(),
                signal.get_value(),
                signal.get_status(),
                sig_age_ms,
                json,
            );
        }
    }
}
