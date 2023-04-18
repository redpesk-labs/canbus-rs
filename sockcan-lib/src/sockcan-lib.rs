/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]
extern crate bitflags;

#[cfg(feature = "with-serde")]
extern crate serde;
#[cfg(feature = "with-serde")]
extern crate serde_derive;

#[path = "./cglue-mod.rs"]
mod cglue;

#[path = "./utils-mod.rs"]
mod utils;

#[path = "./socket-mod.rs"]
mod socket;

#[path = "./dbcpool-mod.rs"]
mod dbcpool;

pub mod prelude {
    pub use socket::*;
    pub use utils::*;
    pub use dbcpool::*;
}