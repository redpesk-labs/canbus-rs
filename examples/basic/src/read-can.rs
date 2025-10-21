/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 */
extern crate sockcan;
use env_logger::Env;
use sockcan::prelude::*;

fn main() -> Result<(), String> {
    // Initialize logging backend for the `log` facade (idempotent).
    let env = Env::default().default_filter_or("info");
    let _ = env_logger::Builder::from_env(env).format_timestamp_millis().try_init();

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

    log::info!("Waiting for Raw CAN package");
    loop {
        let frame = sockfd.get_can_frame();
        let frame_stamp = frame.get_stamp();
        let frame_data = frame.get_data();
        match frame.get_raw() {
            CanAnyFrame::RawFd(frame) => log::info!(
                "Received FdFrame id:{:#04x} stamp:{} len:{} data:{:?}",
                frame.get_id(),
                frame_stamp,
                frame.get_len(),
                frame_data
            ),
            CanAnyFrame::RawStd(frame) => log::info!(
                "Received StdFrame id:{:#04x} stamp:{}, len:{} data:{:?}",
                frame.get_id(),
                frame_stamp,
                frame.get_len(),
                frame_data
            ),
            CanAnyFrame::Err(error) => {
                return Err(format!("fail reading candev: {error}"));
            },
            CanAnyFrame::None(canid) => log::debug!("Got timeout canid:{canid}"),
        }
    }
}
