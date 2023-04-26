/*
 * Copyright (C) 2018 Marcel Buesing (MIT License)
 * Origin: https://github.com/marcelbuesing/can-dbc
 *
 * Adaptation (2022) to Redpesk and LibAfb model
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

#![doc(html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png", html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico")]
extern crate nom;
extern crate heck;
extern crate sockcan;

#[path = "./dbc-parser.rs"]
mod parser;

#[path = "./dbc-data.rs"]
mod data;

#[path = "./dbc-gencode.rs"]
mod gencode;

// ! #[cfg(test)]
// #[path = "./dbc-test.rs"]
// pub mod dbc_test;

pub mod prelude {
    pub use data::*;
    pub use gencode::*;
}