/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 */

extern crate dbcparser;
use dbcparser::prelude::*;

use std::env;
use std::io::{self,Error,ErrorKind};

const HEADER:&str = "
// -----------------------------------------------------------------------
//              <- DBC file Rust mapping ->
// -----------------------------------------------------------------------
//  Do not exit this file it will be regenerated automatically by cargo.
//  Check:
//   - build.rs at project root for dynamically mapping
//   - example/demo/dbc-log/??? for static values
//  Reference: iot.bzh/Redpesk canbus-rs code generator
// -----------------------------------------------------------------------
";


fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println! ("SYNTAX-Error => can-dbc '/xxx/file.dbc' '/yyyy/candump.log'");
        println! ("example: {} 'examples/dbc-log/model3can.dbc' 'examples/dbc-log/candump.log'", args[0]);
        return Err(Error::new(
                ErrorKind::Other,
                "invalid input arguments"
                ));
    }

    let dbcfile= args[1].as_str();
    let dumpfile= args[2].as_str();

    DbcParser::new("Demo")
        .dbcfile(dbcfile)
        .outfile(dumpfile)
        .header(HEADER)
        .generate() ?;

    Ok(())
}