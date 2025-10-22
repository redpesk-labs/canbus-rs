/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 * Reference:
 *  https://github.com/canboat/canboat
 *  https://github.com/nberlette/canbus.git
 *  https://github.com/iDoka/awesome-canbus
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

    // open j1939 in promiscuous mode
    let mut sock =
        match SockCanHandle::open_j1939(VCAN, SockJ1939Addr::Promiscuous, CanTimeStamp::CLASSIC) {
            Err(error) => return Err(format!("fail opening candev {error}")),
            Ok(value) => value,
        };

    // when using basic/etc/start-pgn129285.sh
    let mut filters = SockJ1939Filters::new();
    match filters
        .add_fast(129_285, 10, 128) // canboat pgn "navigationRouteWpInformation"
        .apply(&sock)
    {
        Err(error) => panic!("j1939-filter fail Error:{error}"),
        Ok(()) => log::info!("sockj1939 filter PGN=129285 ready"),
    }

    // register NMEA user land fast packet protocol on top of J1939
    if filters.get_fastlen() > 0 {
        sock.set_callback(Box::new(filters));
    }

    // choose blocking/non blocking mode [default blocking]
    // sock.set_blocking(true).expect("Fail to set block mode");
    println!("sockj1939 waiting for packet");
    let mut count = 0;
    loop {
        count += 1;

        let frame = sock.get_j1939_frame();
        match frame.get_opcode() {
            SockCanOpCode::RxRead(_data) => println!(
                "({:4}) J1939 pgn:{:#04x}({}) stamp:{} len:{} data:{}",
                count,
                frame.get_pgn(),
                frame.get_pgn(),
                frame.get_stamp(),
                frame.get_len(),
                frame
                    .get_data()
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" "),
            ),
            SockCanOpCode::RxError(error) => {
                log::info!("{error}");
            },
            // if packet is partial just silently ignore it
            _ => {},
        }
    }
}
