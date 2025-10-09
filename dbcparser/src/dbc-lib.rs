/*
 * Copyright (C) 2018 Marcel Buesing (MIT License)
 * Origin: https://github.com/marcelbuesing/can-dbc
 *
 * Adaptation (2022) to Redpesk and LibAfb model
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]

extern crate nom;

// --- Déclarations de modules (tes fichiers s'appellent dbc-*.rs)
#[path = "dbc-data.rs"]
pub mod data;

#[path = "dbc-parser.rs"]
pub mod parser;

#[path = "dbc-gencode.rs"]
pub mod gencode;

// --- Re-exports (optionnels) pour l'API publique
pub use crate::data::*;
pub use crate::gencode::*;
// pub use crate::parser::{dbc_from_str /*, ...*/};

/// Prélude pratique pour `use dbcparser::prelude::*;`
pub mod prelude {
    pub use crate::data::*;
    pub use crate::gencode::*;
    pub use crate::parser::*;
}
