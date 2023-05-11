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
use sockcan::prelude::*;

fn main() -> Result<(), String> {
    const VCAN: &str = "vcan0";

    let sock = match SockCanHandle::open_bcm(VCAN, CanTimeStamp::CLASSIC) {
        Err(error) => return Err(format!("fail opening candev {}", error.to_string())),
        Ok(value) => value,
    };

    // match SockBcmCmd::new(CanBcmOpCode::RxSetup, CanBcmFlag::RX_FILTER_ID, 0x118)
    //     //.add_multiplex(0x01)
    //     //.add_multiplex(0x02)
    //     .apply(&sock)
    // {
    //     Err(error) => panic!("bcm-filter fail Error:{}", error.to_string()),
    //     Ok(()) => println!("sockbcm filter ready"),
    // }

    // set a filter on a non existing message
    match SockBcmCmd::new(
        CanBcmOpCode::RxSetup,
        CanBcmFlag::RX_FILTER_ID | CanBcmFlag::SET_TIMER | CanBcmFlag::START_TIMER | CanBcmFlag::RX_ANNOUNCE_RESUME,
        0x118,
    )
    .set_timers(500, 1000)
    .apply(&sock)
    {
        Err(error) => panic!("bcm-filter fail Error:{}", error.to_string()),
        Ok(()) => println!("sockbcm filter ready"),
    }


    match SockBcmCmd::new(
        CanBcmOpCode::RxSetup,
        CanBcmFlag::RX_FILTER_ID | CanBcmFlag::SET_TIMER | CanBcmFlag::START_TIMER,
        0x257,
    )
    .set_timers(500, 1000)
    .apply(&sock)
    {
        Err(error) => return Err(format!("bcm-filter fail Error {}", error.to_string())),
        Ok(()) => println!("sockbcm filter ready"),
    }

    // choose blocking/non blocking mode [default blocking]
    // sock.set_blocking(true).expect("Fail to set block mode");
    let mut count = 0;
    loop {
        count += 1;

        let msg = sock.get_bcm_frame();
        match msg.get_opcode() {
            CanBcmOpCode::RxChanged => match msg.get_raw() {
                CanAnyFrame::RawFd(frame) => println!(
                    "{:4} BCM new FdFrame contend canid:{:#04x} stamp:{} len:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                    frame.get_len()
                ),
                CanAnyFrame::RawStd(frame) => println!(
                    "{:4} BCM new Frame contend canid:{:#04x}({}) stamp:{}",
                    count,
                    frame.get_id(),
                    frame.get_id(),
                    msg.get_stamp(),
                ),
                CanAnyFrame::None(canid) => panic!("{:4} Frame timeout canid:{}", count, *canid),

                CanAnyFrame::Err(error) => {
                    panic!("Fail reading candev Error:{}", error.to_string())
                }
            },

            CanBcmOpCode::RxRead => match msg.get_raw() {
                CanAnyFrame::RawFd(frame) => println!(
                    "{} BCM FdFrame canid:{:#04x} stamp:{} len:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                    frame.get_len()
                ),
                CanAnyFrame::RawStd(frame) => println!(
                    "{} BCM FdFrame canid:{:#04x} stamp:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                ),

                CanAnyFrame::None(canid) => println!("Got timeout canid:{}", canid),

                CanAnyFrame::Err(error) => {
                    panic!("Fail reading candev Error:{}", error.to_string())
                }
            },

            CanBcmOpCode::RxStatus => println!("{:4} BCM status filter canid:{:#04x}", count, msg.get_id().unwrap()),
            CanBcmOpCode::RxSetup => println!("{:4} BCM setup filter canid:{:#04x}", count, msg.get_id().unwrap()),
            CanBcmOpCode::RxTimeout => println!("{:4} BCM Timeout canid:{:#04x}", count, msg.get_id().unwrap()),
            _ => panic!("unsupported bcm opcode:{:?}", msg.get_opcode()),
        }
    }
}
