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

fn main() -> Result<(), String> {
    const VCAN: &str = "vcan0";

    let sockfd = match SockCanHandle::open_raw(VCAN, CanTimeStamp::CLASSIC) {
        Err(error) => return Err(format!("fail opening candev {error}")),
        Ok(value) => value,
    };

    if let Err(error) = SockCanFilter::new(0)
        .add_whitelist(0x257, &FilterMask::SFF_MASK)
        .add_whitelist(0x118, &(FilterMask::ERR_FLAG | FilterMask::SFF_MASK))
        .apply(&sockfd)
    {
        return Err(format!("raw-filter fail filter Error:{error}"));
    }

    // check a full frame
    let frame = sockfd.get_can_frame();
    let frame_id = frame.get_id().unwrap();
    let frame_len = frame.get_len().unwrap();
    let frame_stamp = frame.get_stamp();
    let frame_data = frame.get_data();
    let frame_source = sockfd.get_ifname(frame.get_iface()).unwrap();
    println!(
        "Received FdFrame id:{frame_id:#04x} stamp:{frame_stamp} source:{frame_source} \
        len:{frame_len} data:{frame_data:?}"
    );

    println!("Waiting for Raw CAN package");
    loop {
        let frame = sockfd.get_can_frame();
        let frame_stamp = frame.get_stamp();
        let frame_data = frame.get_data();
        match frame.get_raw() {
            CanAnyFrame::RawFd(frame) => println!(
                "Received FdFrame id:{:#04x} stamp:{} len:{} data:{:?}",
                frame.get_id(),
                frame_stamp,
                frame.get_len(),
                frame_data
            ),
            CanAnyFrame::RawStd(frame) => println!(
                "Received StdFrame id:{:#04x} stamp:{}, len:{} data:{:?}",
                frame.get_id(),
                frame_stamp,
                frame.get_len(),
                frame_data
            ),
            CanAnyFrame::Err(error) => panic!("Fail reading candev Error:{error}"),
            CanAnyFrame::None(canid) => println!("Got timeout canid:{canid}"),
        }
    }
}
