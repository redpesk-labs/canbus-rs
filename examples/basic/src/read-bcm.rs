/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 *  ref: https://github.com/commaai/opendbc
 */

extern crate sockcan;
use env_logger::Env;
use sockcan::prelude::*;

fn main() -> Result<(), String> {
    // Initialize logging backend for the `log` facade (idempotent).
    let env = Env::default().default_filter_or("info");
    let _ = env_logger::Builder::from_env(env).format_timestamp_millis().try_init();

    const VCAN: &str = "vcan0";

    let sock = match SockCanHandle::open_bcm(VCAN, CanTimeStamp::CLASSIC) {
        Err(error) => return Err(format!("fail opening candev {error}")),
        Ok(value) => value,
    };

    //bcm rx subscriptions for 0x118 and 0x257
    //This section documents the two Broadcast Manager (BCM) RX subscriptions configured in your code.
    //Each call creates an independent subscription on the same BCM socket.

    // set a filter on a non existing message

    // RX_FILTER_ID flag ties each subscription to the exact CAN identifier given as the third argument
    // SET_TIMER and START_TIMER enable and start per-subscription timers.
    // RX_ANNOUNCE_RESUME is used on the first subscription to trigger an immediate state/update after (re)configuration;
    match SockBcmCmd::new(
        CanBcmOpCode::RxSetup,
        CanBcmFlag::RX_FILTER_ID
            | CanBcmFlag::SET_TIMER
            | CanBcmFlag::START_TIMER
            | CanBcmFlag::RX_ANNOUNCE_RESUME,
        0x118,
    )
    .set_timers(500, 1000)
    .apply(&sock)
    {
        Err(error) => panic!("bcm-filter fail Error:{error}"),
        Ok(()) => log::info!("sockbcm filter ready canid: 0x118"),
    }

    match SockBcmCmd::new(
        CanBcmOpCode::RxSetup,
        CanBcmFlag::RX_FILTER_ID | CanBcmFlag::SET_TIMER | CanBcmFlag::START_TIMER,
        0x257,
    )
    .set_timers(500, 1000)
    .apply(&sock)
    {
        Err(error) => return Err(format!("bcm-filter fail Error {error}")),
        Ok(()) => log::info!("sockbcm filter ready canid: 0x257"),
    }

    // choose blocking/non blocking mode [default blocking]
    // sock.set_blocking(true).expect("Fail to set block mode");
    let mut count = 0;
    loop {
        count += 1;

        let msg = sock.get_bcm_frame();
        match msg.get_opcode() {
            CanBcmOpCode::RxChanged => match msg.get_raw() {
                CanAnyFrame::RawFd(frame) => log::info!(
                    "{:4} BCM new FdFrame contend canid:{:#04x} stamp:{} len:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                    frame.get_len()
                ),
                CanAnyFrame::RawStd(frame) => log::info!(
                    "{:4} BCM new Frame contend canid:{:#04x}({}) stamp:{}",
                    count,
                    frame.get_id(),
                    frame.get_id(),
                    msg.get_stamp()
                ),
                CanAnyFrame::None(canid) => panic!("{:4} Frame timeout canid:{}", count, *canid),

                CanAnyFrame::Err(error) => {
                    panic!("Fail reading candev Error:{error}")
                },
            },
            CanBcmOpCode::RxRead => match msg.get_raw() {
                CanAnyFrame::RawFd(frame) => log::info!(
                    "{} BCM FdFrame canid:{:#04x} stamp:{} len:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                    frame.get_len()
                ),
                CanAnyFrame::RawStd(frame) => log::info!(
                    "{} BCM FdFrame canid:{:#04x} stamp:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                ),
                CanAnyFrame::None(canid) => log::info!("Got timeout canid:{canid}"),
                CanAnyFrame::Err(error) => {
                    panic!("Fail reading candev Error:{error}")
                },
            },
            CanBcmOpCode::RxStatus => {
                log::info!("{count:4} BCM status filter canid:{:#04x}", msg.get_id().unwrap());
            },
            CanBcmOpCode::RxSetup => {
                log::info!("{count:4} BCM setup filter canid:{:#04x}", msg.get_id().unwrap());
            },
            CanBcmOpCode::RxTimeout => {
                log::info!("{count:4} BCM Timeout canid:{:#04x}", msg.get_id().unwrap());
            },
            _ => panic!("unsupported bcm opcode:{:?}", msg.get_opcode()),
        }
    }
}
