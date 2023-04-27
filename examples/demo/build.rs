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

fn main() {

    // !!! WARNING: model3can.dbc generate 33000 lines of code !!!
    //let dbc_infile="../dbc-log/simple.dbc";
    let dbc_infile="../dbc-log/model3can.dbc";

    // generate parser outside of project git repo
    let dbc_outfile="/tmp/dbc-autogen.rs";

    // invalidate build when dbc file changes
    println!("cargo:rerun-if-changed={}", dbc_infile);

    let header = "
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
    DbcParser::new("DbcSimple")
        .dbcfile(dbc_infile)
        .outfile(dbc_outfile)
        .header(header)
        .range_check(true)
        .whitelist(vec![280,614,599]) // restrict generated code size to candump.log messages
        //.blacklist(vec![614, 599])
        .generate()
        .expect("Fail to parse dbc-file'\n");
}
