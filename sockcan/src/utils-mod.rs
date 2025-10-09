/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
*/

use super::cglue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CanError {
    uid: String,
    info: String,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug)]
pub enum CanBcmOpCode {
    TxSetup,
    TxDelete,
    TxRead,
    TxSend,
    TxStatus,
    TxExpired,
    RxSetup,
    RxDelete,
    RxRead,
    RxStatus,
    RxTimeout,
    RxChanged,
    Unknown,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CanMsgData<'a> {
    pub canid: u32,
    pub len: u8,
    pub stamp: u64,
    pub opcode: CanBcmOpCode,
    pub data: &'a [u8],
}

impl Clone for CanError {
    fn clone(&self) -> CanError {
        CanError { uid: self.uid.clone(), info: self.info.clone() }
    }
}

pub trait MakeError<T> {
    fn make(uid: &str, msg: T) -> CanError;
}

impl MakeError<&str> for CanError {
    fn make(uid: &str, msg: &str) -> CanError {
        CanError { uid: uid.to_string(), info: msg.to_string() }
    }
}

impl MakeError<String> for CanError {
    fn make(uid: &str, msg: String) -> CanError {
        CanError { uid: uid.to_string(), info: msg }
    }
}

impl CanError {
    pub fn new<T>(uid: &str, msg: T) -> CanError
    where
        CanError: MakeError<T>,
    {
        Self::make(uid, msg)
    }
    #[must_use]
    pub fn get_uid(&self) -> String {
        self.uid.clone()
    }
    #[must_use]
    pub fn get_info(&self) -> String {
        self.info.clone()
    }
}

impl fmt::Display for CanError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        write!(format, "uid:{} info:{}", self.uid, self.info)
    }
}

impl fmt::Debug for CanError {
    fn fmt(&self, format: &mut fmt::Formatter) -> fmt::Result {
        write!(format, "uid:{} info:{}", self.uid, self.info)
    }
}

/// Returns current time formatted with `format`.
///
/// # Errors
/// Returns `CanError` if time formatting fails or the system clock is unavailable.
pub fn get_time(format: &str) -> Result<String, CanError> {
    match cglue::get_time(format) {
        Err(()) => Err(CanError {
            uid: "invalid-date-format".to_string(),
            info: "check linux strftime api".to_string(),
        }),
        Ok(date) => Ok(date),
    }
}
