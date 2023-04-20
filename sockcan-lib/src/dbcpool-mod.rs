/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
*/
use std::fmt;
use std::any::Any;
use utils::*;


#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CanDataStatus {
    Timeout,
    Updated,
    Unchanged,
    Error,
    Unset,
}

impl  fmt::Display for CanDataStatus {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status= match self {
            CanDataStatus::Timeout => "Timeout",
            CanDataStatus::Updated => "Updated",
            CanDataStatus::Unchanged => "Unchanged",
            CanDataStatus::Error => "Error",
            CanDataStatus::Unset => "Unset",
        };
        write! (format, "{}", status)
    }
}

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

impl  fmt::Display for CanDbcType {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text= match self {
            CanDbcType::U8(val) => format!("{})", val),
            CanDbcType::U16(val) => format!("{}", val),
            CanDbcType::U32(val) => format!("{}", val),
            CanDbcType::U64(val) => format!("{}", val),
            CanDbcType::I8(val) => format!("{}", val),
            CanDbcType::I16(val) => format!("{}", val),
            CanDbcType::I32(val) => format!("{}", val),
            CanDbcType::I64(val) => format!("{}", val),
            CanDbcType::Bool(val) => format!("{}", val),
            CanDbcType::F64(val) => format!("{}", val),
        };
        fmt.pad(&text)
    }
}

impl fmt::Debug for CanDbcType {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text= match self {
            CanDbcType::U8(val) => format!("{:<8}  (u8)", val),
            CanDbcType::U16(val) => format!("{:<8} (u16)", val),
            CanDbcType::U32(val) => format!("{:<8} (u32)", val),
            CanDbcType::U64(val) => format!("{:<8} (u64)", val),
            CanDbcType::I8(val) => format!("{:<8}   (i8)", val),
            CanDbcType::I16(val) => format!("{:<8} (i16)", val),
            CanDbcType::I32(val) => format!("{:<8} (i32)", val),
            CanDbcType::I64(val) => format!("{:<6} (i64)", val),
            CanDbcType::Bool(val) => format!("{:<8}(bool)", val),
            CanDbcType::F64(val) => format!("{:<8.3} (f64)", val),
        };
            fmt.debug_struct(&text)
            .finish()
        }
}
pub trait FromCanDbcType<T> {
    fn convert(value: &CanDbcType) -> Result<T, ()>;
}

impl CanDbcType {
    pub fn cast<T>(&self) -> Result<T, CanError>
    where CanDbcType: FromCanDbcType<T> {
        match Self::convert(self) {
            Ok(val) => Ok(val),
            Err(()) => Err(CanError::new ("invalid-type-cast","requested type is invalid")),

        }
    }
}

pub use to_can_type;
#[macro_export]
macro_rules! to_can_type {
    ($src:ty, $dst:tt) => {
        impl From<$src> for CanDbcType {
            fn from(value: $src) -> Self {
                CanDbcType::$dst(value)
            }
        }
        impl FromCanDbcType<$src> for CanDbcType {
            fn convert(value: &CanDbcType) -> Result<$src, ()> {
                match value {
                    CanDbcType::$dst(data) => return Ok(*data),
                    _ => Err(()),
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

pub trait CanDbcSignal {
    fn get_value(&self) -> CanDbcType;
    fn set_value(&mut self, value: CanDbcType, data: &mut [u8]) -> Result<(), CanError>;
    fn get_name(&self) -> &'static str;
    fn get_stamp(&self) -> u64;
    fn get_status(&self) -> CanDataStatus;
    fn update(&mut self, frame: &CanMsgData);
    fn as_any(&mut self) -> &mut dyn Any;
}

pub trait CanDbcMessage {
    fn get_id(&self) -> u32;
    fn update(&mut self, data: &CanMsgData);
    fn get_stamp(&self) -> u64;
    fn get_status(&self) -> CanBcmOpCode;
    fn get_name(&self) -> &'static str;
    fn get_signals(&mut self) -> &mut [Box<dyn CanDbcSignal>];
    fn as_any(&mut self) -> &mut dyn Any;
}

pub trait CanDbcPool {
    fn new(uid: &'static str) -> Self;
    fn get_ids(&self) -> &'static [u32];
    fn update(&self, data: &CanMsgData) -> Result<&mut dyn CanDbcMessage, CanError>;
}
