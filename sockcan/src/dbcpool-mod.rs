/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
*/
use crate::prelude::*;
use std::any::Any;
use std::cell::{RefCell, RefMut};
use std::fmt;
use std::rc::Rc;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::utils::CanError; // bring error type into scope for Result<T, CanError>

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CanDataStatus {
    Timeout,
    Updated,
    Unchanged,
    Error,
    Unset,
}

impl fmt::Display for CanDataStatus {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            CanDataStatus::Timeout => "Timeout",
            CanDataStatus::Updated => "Updated",
            CanDataStatus::Unchanged => "Unchanged",
            CanDataStatus::Error => "Error",
            CanDataStatus::Unset => "Unset",
        };
        write!(format, "{status}" )
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CanDbcType {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F64(f64),
    Bool(bool),
}

impl fmt::Display for CanDbcType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            CanDbcType::U8(val) => format!("{val})"),
            CanDbcType::U16(val) => format!("{val}"),
            CanDbcType::U32(val) => format!("{val}"),
            CanDbcType::U64(val) => format!("{val}"),
            CanDbcType::I8(val) => format!("{val}"),
            CanDbcType::I16(val) => format!("{val}"),
            CanDbcType::I32(val) => format!("{val}"),
            CanDbcType::I64(val) => format!("{val}"),
            CanDbcType::Bool(val) => format!("{val}"),
            CanDbcType::F64(val) => format!("{val}"),
        };
        fmt.pad(&text)
    }
}

impl fmt::Debug for CanDbcType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            CanDbcType::U8(val) => format!("{val}(u8)"),
            CanDbcType::U16(val) => format!("{val}(u16)"),
            CanDbcType::U32(val) => format!("{val}(u32)"),
            CanDbcType::U64(val) => format!("{val}(u64)"),
            CanDbcType::I8(val) => format!("{val}(i8)"),
            CanDbcType::I16(val) => format!("{val}(i16)"),
            CanDbcType::I32(val) => format!("{val}(i32)"),
            CanDbcType::I64(val) => format!("{val}(i64)"),
            CanDbcType::Bool(val) => format!("{val}(bool)"),
            CanDbcType::F64(val) => format!("{val}(f64)"),
        };
        fmt.debug_struct(&text).finish()
    }
}
pub trait FromCanDbcType<T> {
    /// Attempts to convert a `CanDbcType` into `T`.
    ///
    /// # Errors
    /// Returns a [`CanError`] when the value cannot be converted to `T`, e.g.:
    /// - the underlying variant does not match the expected type;
    /// - the payload size/endianness is incompatible;
    /// - numeric conversion would overflow/underflow.
    fn convert(value: &CanDbcType) -> Result<T, CanError>;
}

impl CanDbcType {
    /// Attempts to cast this dynamic value to `T`.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `CanDbcType` cannot be converted to `T`.
    pub fn cast<T>(&self) -> Result<T, CanError>
    where
        CanDbcType: FromCanDbcType<T>,
    {
        match Self::convert(self) {
            Ok(val) => Ok(val),
            Err(e)  => Err(e), // propagate the CanError from convert(...)
        }
    }
}

#[macro_export]
macro_rules! to_can_type {
    ($src:ty, $variant:ident) => {
        impl FromCanDbcType<$src> for CanDbcType {
            #[inline]
            fn convert(value: &CanDbcType) -> Result<$src, CanError> {
                match value {
                    CanDbcType::$variant(v) => Ok(*v),
                    // Fallback error when the underlying variant doesn't match
                    _ => Err(CanError::new(
                        "dbc-convert",
                        concat!("expected variant ", stringify!($variant)),
                    )),
                }
            }
        }
    };
}
// generate rust type to can-type converters
to_can_type!(u8, U8);
to_can_type!(u16, U16);
to_can_type!(u32, U32);
to_can_type!(u64, U64);
to_can_type!(i8, I8);
to_can_type!(i16, I16);
to_can_type!(i32, I32);
to_can_type!(i64, I64);
to_can_type!(bool, Bool);
to_can_type!(f64, F64);

pub trait CanSigCtrl {
    fn sig_notification(&self, sig: &dyn CanDbcSignal) -> i32;
}
pub trait CanDbcSignal {
    fn get_value(&self) -> CanDbcType;
    /// Encodes `value` into `data`.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` does not fit into the target layout or if
    /// the destination buffer is too small / misaligned.
    fn set_value(&mut self, value: CanDbcType, data: &mut [u8]) -> Result<(), CanError>;
    fn get_name(&self) -> &'static str;
    fn get_stamp(&self) -> u64;
    fn get_status(&self) -> CanDataStatus;
    fn update(&mut self, frame: &CanMsgData) -> i32;
    fn as_any(&mut self) -> &mut dyn Any;
    fn to_json(&self) -> String;
    fn reset(&mut self);
    fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>);
}

pub trait CanMsgCtrl {
    fn msg_notification(&self, msg: &dyn CanDbcMessage);
}

pub trait CanDbcMessage {
    fn get_id(&self) -> u32;
    /// Updates the pool with a newly received CAN frame.
    ///
    /// The implementation is expected to decode `data` and update the
    /// corresponding message entry (e.g., signals, timestamps, caches).
    ///
    /// # Errors
    /// Returns a `CanError` if the update cannot be applied, for example:
    /// - the CAN ID is unknown / not registered in the pool;
    /// - the frame length is invalid for the expected message format;
    /// - decoding/parsing of one or more signals fails (range/overflow, endianness, etc.);
    /// - there is an outstanding borrow that prevents a mutable update (e.g., `RefCell` borrow conflict);
    /// - any underlying I/O or memory error occurs while updating internal buffers.
    fn update(&mut self, data: &CanMsgData) -> Result<(), CanError>;
    fn get_stamp(&self) -> u64;
    fn get_status(&self) -> CanBcmOpCode;
    fn get_name(&self) -> &'static str;
    fn get_signals(&self) -> &[Rc<RefCell<Box<dyn CanDbcSignal>>>];
    fn as_any(&mut self) -> &mut dyn Any;
    /// Resets the internal state of the pool (counters, buffers, caches, etc.).
    ///
    /// # Errors
    /// Returns a `CanError` if the reset fails, for example:
    /// - when there are outstanding borrows (e.g., active `RefCell` borrows);
    /// - when an underlying resource (memory, socket, etc.) cannot be reinitialized.
    fn reset(&mut self) -> Result<(), CanError>;
    fn set_callback(&mut self, callback: Box<dyn CanMsgCtrl>);
    fn get_listeners(&self) -> i32;
}


pub trait CanDbcPool {
    /// Returns the list of known CAN IDs handled by this pool.
    fn get_ids(&self) -> &[u32];

    /// Returns the decoded messages managed by this pool.
    fn get_messages(&self) -> &[Rc<RefCell<Box<dyn CanDbcMessage>>>];

    /// Returns a mutable reference to the message buffer for `canid`.
    ///
    /// # Errors
    ///
    /// Returns an error if `canid` is not registered in this pool or
    /// if the message cannot be borrowed mutably at this time.
    fn get_mut(&self, canid: u32) -> Result<RefMut<'_, Box<dyn CanDbcMessage>>, CanError>;

    /// Updates the pool with the provided raw CAN frame.
    ///
    /// On success, returns a mutable reference to the updated message.
    ///
    /// # Errors
    ///
    /// Returns an error if the frame ID is unknown, the payload size
    /// is invalid for the expected message, or decoding fails.
    fn update(&self, data: &CanMsgData) -> Result<RefMut<'_, Box<dyn CanDbcMessage>>, CanError>;
}