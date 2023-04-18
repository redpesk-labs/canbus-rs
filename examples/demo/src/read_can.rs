/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 */
extern crate sockcan;

//use bitvec::prelude::*;
use sockcan::prelude::*;


fn main() -> Result <(), String> {
    const VCAN: &str = "vcan0";

    let sockfd = match SockCanHandle::open_raw(VCAN, CanTimeStamp::CLASSIC) {
        Err(error) => return Err(format!("fail opening candev {}", error.to_string())),
        Ok(value) => value,
    };

    match SockCanFilter::new(0)
        .add_whitelist(0x257, FilterMask::SFF_MASK)
        .add_whitelist(0x118, FilterMask::ERR_FLAG | FilterMask::SFF_MASK)
        .apply(&sockfd)
    {
        Err(error) => return Err(format!("raw-filer fail filter Error:{}", error.to_string())),
        Ok(()) => {}
    }

    // check a full frame
    let frame= sockfd.get_can_frame();
    let frame_id= frame.get_id().unwrap();
    let frame_len= frame.get_len().unwrap();
    let frame_stamp=frame.get_stamp();
    let frame_source= frame.get_ifname(&sockfd).unwrap();
    println!("Received FdFrame id:{:#04x} stamp:{} source:{} len:{}", frame_id, frame_stamp, frame_source, frame_len);

    println! ("Waiting for Raw CAN package");
    loop {
        let frame= sockfd.get_can_frame();
        let frame_stamp=frame.get_stamp();
        match frame.get_raw() {
            CanAnyFrame::RawFd(frame) => println!("Received FdFrame id:{:#04x} stamp:{} len:{}", frame.get_id(), frame_stamp, frame.get_len()),
            CanAnyFrame::RawStd(frame) => println!("Received StdFrame id:{:#04x} stamp:{}", frame.get_id(), frame_stamp),
            CanAnyFrame::Err(error) => panic!("Fail reading candev Error:{}", error.to_string()),
        }
    };
}
