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
//use sockcan::prelude::*;

use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
//use std::thread::sleep;
//use std::time::{Duration, Instant};
//use sockcan::prelude::CanFrameRaw;
use sockcan::prelude::generate_frame;
use sockcan::prelude::CanTimeStamp;
use sockcan::prelude::SockCanHandle;
use std::convert::TryInto;

//use sockcan::{CanTimeStamp, SockCanHandle, SockCanMsg};

#[derive(Debug, Clone)]
struct ParsedCan2 {
    id: u32,       // identifiant CAN (hex)
    data: Vec<u8>, // payload
}

fn parse_line(line: &str, re: &Regex) -> Option<ParsedCan2> {
    // captures: 1=ts  2=iface  3=id_hex  4=data_hex
    let caps = re.captures(line)?;

    // id en hex sans 0x, peut être 11-bit ou 29-bit; on retire un éventuel bit extended si présent dans ta source
    let id = u32::from_str_radix(caps.get(3)?.as_str(), 16).ok()?;
    // si besoin: id &= 0x1FFF_FFFF; // pour forcer 29 bits utiles

    let data_hex = caps.get(4)?.as_str();
    if data_hex.len() % 2 != 0 {
        return None;
    }
    let mut data = Vec::with_capacity(data_hex.len() / 2);
    for i in (0..data_hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&data_hex[i..i + 2], 16).ok()?;
        data.push(byte);
    }

    Some(ParsedCan2 { id, data })
}

/// Charge et parse un fichier can-dump
fn load_dump<P: AsRef<Path>>(path: P) -> Result<Vec<ParsedCan2>, String> {
    let file = File::open(path.as_ref()).map_err(|e| format!("open dump failed: {e}"))?;
    let reader = BufReader::new(file);

    // Exemple de lignes visées :
    // (1597243671.759299) elmcan 118#5906221800000000
    let re = Regex::new(
        r"^\((?P<ts>\d+\.\d+)\)\s+(?P<iface>\S+)\s+(?P<id>[0-9A-Fa-f]+)#(?P<data>[0-9A-Fa-f]+)\s*$",
    )
    .map_err(|e| format!("regex build failed: {e}"))?;

    let mut out = Vec::new();
    for (lineno, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("read line {lineno}: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some(parsed) = parse_line(&line, &re) {
            out.push(parsed);
        } else {
            return Err(format!("parse error at line {}: {}", lineno + 1, line));
        }
    }
    Ok(out)
}

fn to_fixed_8_exact(v: Vec<u8>) -> Result<[u8; 8], String> {
    v.try_into()
        .map_err(|vv: Vec<u8>| format!("expected 8 bytes, got {}", vv.len()))
}

fn main() -> Result<(), String> {
    // Initialize logging backend for the `log` facade (idempotent).
    let env = Env::default().default_filter_or("info");
    let _ = env_logger::Builder::from_env(env).format_timestamp_millis().try_init();

    let mut args = std::env::args().skip(1);
    let dump_path = args.next().ok_or("missing dump path (e.g., dump.log)")?;

    const VCAN: &str = "vcan0";
    let iface = args.next().ok_or("missing can iface (e.g., vcan0)")?;

    let _no_timing = args.any(|a| a == "--no-timing");

    let records = load_dump(&dump_path)?;
    println!("loaded {} frames from {}", records.len(), dump_path);

    // true = respecte les timestamps relatifs, false = envoi au plus vite
    //replay_dump_on(&iface, &records, !no_timing)?;

    log::info!("replay done on {}", iface);

    let sockfd = match SockCanHandle::open_raw(VCAN, CanTimeStamp::CLASSIC) {
        Err(error) => return Err(format!("fail opening candev {error}")),
        Ok(value) => value,
    };

    for rec in records.iter() {
        let data8 = to_fixed_8_exact(rec.data.clone())?;
        let f_std = generate_frame(rec.id, &data8).map_err(|e| e.to_string())?;
        sockfd.write_frame(&f_std).map_err(|e| e.to_string())?;
    }
    Ok(())
}
