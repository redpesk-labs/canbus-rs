
/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 */

extern crate sockcan;

use sockcan::prelude::*;

fn main() {
    const VCAN: &str = "vcan0";

    let sock = match SockCanHandle::open_bcm(VCAN, CanTimeStamp::CLASSIC) {
        Err(error) => panic!("fail opening candev {}", error.to_string()),
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

    match SockBcmCmd::new(
        CanBcmOpCode::RxSetup,
        CanBcmFlag::RX_FILTER_ID | CanBcmFlag::SET_TIMER,
        0x257,
    )
    .set_timers(500, 1000)
    .apply(&sock)
    {
        Err(error) => panic!("bcm-filter fail Error:{}", error.to_string()),
        Ok(()) => println!("sockbcm filter ready"),
    }

    // match SockBcmCmd::new(CanBcmOpCode::RxSetup, CanBcmFlag::RX_FILTER_ID, 0x266).apply(&sock) {
    //     Err(error) => panic!("bcm-filter fail Error:{}", error.to_string()),
    //     Ok(()) => println!("sockbcm filter ready"),
    // }

    // choose blocking/non blocking mode [default blocking]
    // sock.set_blocking(true).expect("Fail to set block mode");
    let mut count = 0;
    loop {
        count += 1;
        let msg = sock.get_bcm_frame();
        match msg.get_opcode() {
            CanBcmOpCode::RxChanged => match msg.get_raw() {
                CanAnyFrame::RawFd(frame) => println!(
                    "{} BCM new FdFrame contend canid:{:#04x} stamp:{} len:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                    frame.get_len()
                ),
                CanAnyFrame::RawStd(frame) => println!(
                    "{} BCM new FdFrame contend canid:{:#04x} stamp:{}",
                    count,
                    frame.get_id(),
                    msg.get_stamp(),
                ),
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
                CanAnyFrame::Err(error) => {
                    panic!("Fail reading candev Error:{}", error.to_string())
                }
            },

            CanBcmOpCode::RxStatus => println!("BCM status filter canid:{}", msg.get_id().unwrap()),
            CanBcmOpCode::RxSetup => println!("BCM setup filter canid:{}", msg.get_id().unwrap()),
            CanBcmOpCode::RxTimeout => println!("BCM Timeout canid:{}", msg.get_id().unwrap()),
            _ => panic!("unsupported bcm opcode:{:?}", msg.get_opcode()),
        }
    }
}
