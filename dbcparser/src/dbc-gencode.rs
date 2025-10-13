/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Source code derivate from: (MIT || Apache2 License)
 *  - https://github.com/technocreatives/dbc-codegen Copyright: Marcel Buesing, Pascal Hertleif, Andres Vahter, ...
 *  - https://github.com/marcelbuesing/can-dbc Copyright: Marcel Buesing (MIT License)
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 * Reference: http://mcu.so/Microcontroller/Automotive/dbc-file-format-documentation_compress.pdf
 */
use crate::data::{
    ByteOrder, DbcObject, Message, MessageId, MultiplexIndicator, SigCodeGen, Signal, Transmitter,
    ValDescription, ValueType, MsgCodeGen,
};
use heck::{ToSnakeCase, ToUpperCamelCase};

use sockcan::prelude::get_time;
use std::fs::File;
use std::io::{self, Error, Write};

const IDT0: &str = "";
const IDT1: &str = "    ";
const IDT2: &str = "        ";
const IDT3: &str = "            ";
const IDT4: &str = "                ";
const IDT5: &str = "                    ";
const IDT6: &str = "                        ";
//const IDT7: &str = "                            ";
pub struct DbcCodeGen {
    outfd: Option<File>,
    dbcfd: DbcObject,
    range_check: bool,
    serde_json: bool,
}

pub struct DbcParser {
    uid: &'static str,
    infile: Option<String>,
    outfile: Option<String>,
    range_check: bool,
    serde_json: bool,
    header: Option<&'static str>,
    whitelist: Option<Vec<u32>>,
    blacklist: Option<Vec<u32>>,
}

const KEYWORDS: [&str; 53] = [
    // https://doc.rust-lang.org/stable/reference/keywords.html
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "union",
    // Internal names
    "_other",
];

#[macro_export]
macro_rules! code_output {
 ($code:ident, $indent:ident, $format:expr, $( $args:expr ),*) => {
    $code.output ($indent,  format! ($format, $($args),*))
 };
 ($code:ident, $indent:ident,$format:expr) => {
    $code.output ($indent, $format)
 }
}

impl ValDescription {
    fn get_type_kamel(&self) -> String {
        if KEYWORDS.contains(&self.b.to_lowercase().as_str())
            || !self.b.starts_with(|c: char| c.is_ascii_alphabetic())
        {
            format!("X{}", self.b).to_upper_camel_case()
        } else {
            // to_upper_camel_case() takes &self; no clone/owned needed
            self.b.to_upper_camel_case()
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_data_value(&self, data: &str) -> String {
        match data {
            "bool" => format!("{}", (self.a as i64) == 1),
            "f64"  => format!("{}_f64", self.a),
            _      => format!("{}_{}", self.a as i64, data),
        }
    }
}

impl Message {
    fn get_type_kamel(&self) -> String {
        if KEYWORDS.contains(&self.name.to_lowercase().as_str())
            || !self.name.starts_with(|c: char| c.is_ascii_alphabetic())
        {
            format!("X{}", self.name).to_upper_camel_case()
        } else {
            self.name.to_upper_camel_case()
        }
    }
}

impl Signal {
    fn le_start_end_bit(&self, msg: &Message) -> io::Result<(u64, u64)> {
        let msg_bits = msg.size.checked_mul(8).unwrap();
        let start_bit = self.start_bit;
        let end_bit = self.start_bit + self.size;

        if start_bit > msg_bits {
            return Err(Error::other(
                format!(
                    "signal:{} starts at {}, but message is only {} bits",
                    self.name, start_bit, msg_bits
                ),
            ));
        }

        if end_bit > msg_bits {
            return Err(Error::other(
                format!("signal:{} ends at {}, but message is only {} bits", self.name, end_bit, msg_bits),
            ));
        }

        Ok((start_bit, end_bit))
    }

    fn be_start_end_bit(self: &Signal, msg: &Message) -> io::Result<(u64, u64)> {
        let result = || -> Option<(u64, u64, u64)> {
            let x = self.start_bit.checked_div(8)?;
            let x = x.checked_mul(8)?;
            let y = self.start_bit.checked_rem(8)?;
            let y = 7u64.checked_sub(y)?;

            let start_bit = x.checked_add(y)?;
            let end_bit = start_bit.checked_add(self.size)?;
            let msg_bits = msg.size.checked_mul(8)?;
            Some((start_bit, end_bit, msg_bits))
        };

        let Some((start_bit, end_bit, msg_bits)) = result() else {
            return Err(Error::other(format!(
                "signal:{} starts at {}, but message is only {} bits",
                self.name, self.start_bit, msg.size
            )));
        };

        if start_bit > msg_bits {
            return Err(Error::other(
                format!("signal:{} starts at {}, but message is only {} bits", self.name, start_bit, msg_bits),
            ));
        }

        if end_bit > msg_bits {
            return Err(Error::other(
                format!(
                    "signal:{} ends at {}, but message is only {} bits",
                    self.name, end_bit, msg_bits
                ),
            ));
        }

        Ok((start_bit, end_bit))
    }

    fn get_data_usize(&self) -> String {
        let size = match self.size {
            n if n <= 8 => "u8",
            n if n <= 16 => "u16",
            n if n <= 32 => "u32",
            _ => "u64",
        };
        size.to_string()
    }

    fn get_data_isize(&self) -> String {
        let size = match self.size {
            n if n <= 8 => "i8",
            n if n <= 16 => "i16",
            n if n <= 32 => "i32",
            _ => "u64",
        };
        size.to_string()
    }

    #[inline]
    fn has_scaling(&self) -> bool {
        const EPS: f64 = 1e-12;
        self.offset.abs() > EPS || (self.factor - 1.0).abs() > EPS
    }

    fn get_data_type(&self) -> String {
        if self.size == 1 {
            "bool".into()
        } else if self.has_scaling() {
            "f64".into()
        } else {
            let size = match self.size {
                n if n <= 8  => "8",
                n if n <= 16 => "16",
                n if n <= 32 => "32",
                _            => "64",
            };
            match self.value_type {
                ValueType::Signed   => format!("i{size}"),
                ValueType::Unsigned => format!("u{size}"),
            }
        }
    }

    fn get_type_kamel(&self) -> String {
        if KEYWORDS.contains(&self.name.to_lowercase().as_str())
            || !self.name.starts_with(|c: char| c.is_ascii_alphabetic())
        {
            format!("X{}", self.name).to_upper_camel_case()
        } else {
            self.name.to_upper_camel_case()
        }
    }

    fn get_type_snake(&self) -> String {
        if KEYWORDS.contains(&self.name.to_lowercase().as_str())
            || !self.name.starts_with(|c: char| c.is_ascii_alphabetic())
        {
            format!("X{}", self.name).to_snake_case()
        } else {
            self.name.to_snake_case()
        }
    }
}


impl SigCodeGen<&DbcCodeGen> for Signal {
    #[allow(clippy::too_many_lines)]
    fn gen_signal_trait(&self, code: &DbcCodeGen, msg: &Message) -> io::Result<()> {
        code_output!(
            code,
            IDT1,
            "/// {}::{} public api (CanDbcSignal trait)",
            msg.get_type_kamel(),
            self.get_type_kamel()
        )?;
        code_output!(code, IDT1, "impl CanDbcSignal for {} {{\n", self.get_type_kamel())?;
        code_output!(code, IDT2, "fn get_name(&self) -> &'static str {")?;
        code_output!(code, IDT3, "self.name")?;
        code_output!(code, IDT2, "}\n")?;

        code_output!(code, IDT2, "fn get_stamp(&self) -> u64 {")?;
        code_output!(code, IDT3, "self.stamp")?;
        code_output!(code, IDT2, "}\n")?;

        code_output!(code, IDT2, "fn get_status(&self) -> CanDataStatus{")?;
        code_output!(code, IDT3, "self.status")?;
        code_output!(code, IDT2, "}\n")?;

        code_output!(code, IDT2, "fn as_any(&mut self) -> &mut dyn Any {")?;
        code_output!(code, IDT3, "self")?;
        code_output!(code, IDT2, "}\n")?;

        //signal update
        code_output!(code, IDT2, "fn update(&mut self, frame: &CanMsgData) -> i32 {")?;

        let read_fn = match self.byte_order {
            ByteOrder::LittleEndian => {
                let (start_bit, end_bit) = self.le_start_end_bit(msg)?;

                format!(
                    "frame.data.view_bits::<Lsb0>()[{start}..{end}].load_le::<{typ}>()",
                    typ = self.get_data_usize(),
                    start = start_bit,
                    end = end_bit,
                )
            }
            ByteOrder::BigEndian => {
                let (start_bit, end_bit) = self.be_start_end_bit(msg)?;

                format!(
                    "frame.data.view_bits::<Msb0>()[{start}..{end}].load_be::<{typ}>()",
                    typ = self.get_data_usize(),
                    start = start_bit,
                    end = end_bit
                )
            }
        };

        code_output!(code, IDT3, "match frame.opcode {")?;
        code_output!(code, IDT4, "CanBcmOpCode::RxChanged => {")?;
        code_output!(code, IDT5, "let value = {};", read_fn)?;

        if self.value_type == ValueType::Signed {
            code_output!(
                code,
                IDT5,
                "let value = {}::from_ne_bytes(value.to_ne_bytes());",
                self.get_data_isize()
            )?;
        }

        if self.size == 1 {
            code_output!(code, IDT5, "self.value= value == 1;")?;
        } else if self.has_scaling() {
            // Scaling is always done on floats
            code_output!(code, IDT5, "let factor = {}_f64;", self.factor)?;
            code_output!(code, IDT5, "let offset = {}_f64;", self.offset)?;
            code_output!(code, IDT5, "let newval= (value as f64) * factor + offset;")?;
            code_output!(code, IDT5, "if newval != self.value {")?;
            code_output!(code, IDT6, "self.value= newval;")?;
            code_output!(code, IDT6, "self.status= CanDataStatus::Updated;")?;
            code_output!(code, IDT6, "self.stamp= frame.stamp;")?;
            code_output!(code, IDT5, "} else {")?;
            code_output!(code, IDT6, "self.status= CanDataStatus::Unchanged;")?;
            code_output!(code, IDT5, "}")?;
        } else {
            code_output!(code, IDT5, "if self.value != value {")?;
            code_output!(code, IDT6, "self.value= value;")?;
            code_output!(code, IDT6, "self.status= CanDataStatus::Updated;")?;
            code_output!(code, IDT6, "self.stamp= frame.stamp;")?;
            code_output!(code, IDT5, "} else {")?;
            code_output!(code, IDT6, "self.status= CanDataStatus::Unchanged;")?;
            code_output!(code, IDT5, "}")?;
        }
        code_output!(code, IDT4, "},")?;
        code_output!(code, IDT4, "CanBcmOpCode::RxTimeout => {")?;
        code_output!(code, IDT5, "self.status=CanDataStatus::Timeout;")?;
        code_output!(code, IDT4, "},")?;
        code_output!(code, IDT4, "_ => {")?;
        code_output!(code, IDT5, "self.status=CanDataStatus::Error;")?;
        code_output!(code, IDT4, "},")?;
        code_output!(code, IDT3, "}")?;

        code_output!(code, IDT3, "match &self.callback {")?;
        code_output!(code, IDT4, "None => 0,")?;
        code_output!(code, IDT4, "Some(callback) => {")?;
        code_output!(code, IDT5, "match callback.try_borrow() {")?;
        code_output!(
            code,
            IDT6,
            "Err(_) => {println!(\"fail to get signal callback reference\"); -1},"
        )?;
        code_output!(code, IDT6, "Ok(cb_ref) => cb_ref.sig_notification(self),")?;
        code_output!(code, IDT5, "}")?;
        code_output!(code, IDT4, "}")?;
        code_output!(code, IDT3, "}")?;
        code_output!(code, IDT2, "}\n")?;

        // signal set_value
        code_output!(
            code,
            IDT2,
            "fn set_value(&mut self, value:CanDbcType, data:&mut [u8]) -> Result<(),CanError> {"
        )?;
        code_output!(code, IDT3, "let value:{}= match value.cast() {{", self.get_data_type())?;
        code_output!(code, IDT4, "Ok(val) => val,")?;
        code_output!(code, IDT4, "Err(error) => return Err(error)")?;
        code_output!(code, IDT3, "};")?;

        code_output!(code, IDT3, "self.set_typed_value(value, data)")?;
        code_output!(code, IDT2, "}\n")?;

        // signal get value
        code_output!(code, IDT2, "fn get_value(&self) -> CanDbcType {")?;
        code_output!(
            code,
            IDT3,
            "CanDbcType::{}(self.get_typed_value())",
            self.get_data_type().to_upper_camel_case()
        )?;
        code_output!(code, IDT2, "}\n")?;

        if code.serde_json {
            code_output!(code, IDT2, "fn to_json(&self) -> String {")?;
            code_output!(code, IDT3, "match serde_json::to_string(self) {")?;
            code_output!(code, IDT4, "Ok(json)=> json,",)?;
            code_output!(code, IDT4, "_ => \"serde-json-error\".to_owned()",)?;
            code_output!(code, IDT3, "}")?;
            code_output!(code, IDT2, "}\n")?;
        }

        // reset signal values
        code_output!(code, IDT2, "fn reset(&mut self) {")?;
        code_output!(code, IDT3, "self.stamp=0;")?;
        code_output!(code, IDT3, "self.reset_value();")?;
        code_output!(code, IDT3, "self.status=CanDataStatus::Unset;")?;
        code_output!(code, IDT2, "}\n")?;

        // set signal notification callback
        code_output!(code, IDT2, "fn set_callback(&mut self, callback: Box<dyn CanSigCtrl>)  {")?;
        code_output!(code, IDT3, "self.callback= Some(RefCell::new(callback));")?;
        code_output!(code, IDT2, "}\n")?;

        code_output!(
            code,
            IDT1,
            "}} // end {}::{} public api\n",
            msg.get_type_kamel(),
            self.get_type_kamel()
        )?;

        Ok(())
    }

    fn gen_dbc_min_max(&self, code: &DbcCodeGen, _msg: &Message) -> io::Result<()> {
        if self.size == 1 {
            return Ok(());
        }

        let typ = self.get_data_type();
        code_output!(
            code,
            IDT2,
            "pub const {}_MIN: {} = {}_{};",
            self.get_type_kamel().to_uppercase(),
            typ,
            self.min,
            typ
        )?;

        code_output!(
            code,
            IDT2,
            "pub const {}_MAX: {} = {}_{};",
            self.get_type_kamel().to_uppercase(),
            typ,
            self.max,
            typ
        )?;
        Ok(())
    }

    fn gen_signal_enum(&self, code: &DbcCodeGen, msg: &Message) -> io::Result<()> {
        if let Some(variants) = code.dbcfd.value_descriptions_for_signal(msg.id, self.name.as_str())
        {
            code_output!(
                code,
                IDT1,
                "// DBC definition for MsgID:{} Signal:{}",
                msg.id.0,
                self.name
            )?;
            if code.serde_json {
                code_output!(code, IDT1, "#[derive(Serialize, Deserialize)]")?;
            }
            code_output!(code, IDT1, "pub enum Dbc{} {{", self.get_type_kamel())?;
            for variant in variants {
                code_output!(code, IDT2, "{},", variant.get_type_kamel())?;
            }
            code_output!(code, IDT2, "_Other({}),", self.get_data_type())?;
            code_output!(code, IDT1, "}\n")?;

            code_output!(
                code,
                IDT1,
                "impl From<Dbc{}> for {} {{",
                self.get_type_kamel(),
                self.get_data_type()
            )?;
            code_output!(
                code,
                IDT2,
                "fn from (val: Dbc{}) -> {} {{",
                self.get_type_kamel(),
                self.get_data_type()
            )?;
            code_output!(code, IDT3, "match val {")?;
            for variant in variants {
                if variant.a > self.max || variant.a < self.min {
                    code_output!(
                        code,
                        IDT4,
                        "Dbc{}::{} => panic! (\"(Hoops) impossible conversion {} -> {}\"),",
                        self.get_type_kamel(),
                        variant.get_type_kamel(),
                        variant.get_data_value(&self.get_data_type()),
                        self.get_data_type()
                    )?;
                } else {
                    code_output!(
                        code,
                        IDT4,
                        "Dbc{}::{} => {},",
                        self.get_type_kamel(),
                        variant.get_type_kamel(),
                        variant.get_data_value(&self.get_data_type())
                    )?;
                }
            }
            code_output!(code, IDT4, "Dbc{}::_Other(x) => x", self.get_type_kamel())?;
            code_output!(code, IDT3, "}")?;
            code_output!(code, IDT2, "}")?;
            code_output!(code, IDT1, "}\n")?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    fn gen_signal_impl(&self, code: &DbcCodeGen, msg: &Message) -> io::Result<()> {
        // signal comments and metadata
        code_output!(code, IDT1, "/// {}::{}", msg.get_type_kamel(), self.get_type_kamel())?;
        if let Some(comment) = code.dbcfd.signal_comment(msg.id, self.name.as_str()) {
            code_output!(code, IDT1, "///")?;
            for line in comment.trim().lines() {
                code_output!(code, IDT1, "/// {}", line)?;
            }
        }
        code_output!(code, IDT1, "/// - Min: {}", self.min)?;
        code_output!(code, IDT1, "/// - Max: {}", self.max)?;
        code_output!(code, IDT1, "/// - Unit: {:?}", self.unit)?;
        code_output!(code, IDT1, "/// - Receivers: {}", self.receivers.join(", "))?;
        code_output!(code, IDT1, "/// - Start bit: {}", self.start_bit)?;
        code_output!(code, IDT1, "/// - Signal size: {} bits", self.size)?;
        code_output!(code, IDT1, "/// - Factor: {}", self.factor)?;
        code_output!(code, IDT1, "/// - Offset: {}", self.offset)?;
        code_output!(code, IDT1, "/// - Byte order: {:?}", self.byte_order)?;
        code_output!(code, IDT1, "/// - Value type: {:?}", self.value_type)?;
        if code.serde_json {
            code_output!(code, IDT1, "#[derive(Serialize, Deserialize)]")?;
        }
        code_output!(code, IDT1, "pub struct {} {{", self.get_type_kamel())?;
        if code.serde_json {
            code_output!(code, IDT2, "#[serde(skip)]")?;
        }
        code_output!(code, IDT2, "callback: Option<RefCell<Box<dyn CanSigCtrl>>>,")?;
        code_output!(code, IDT2, "status: CanDataStatus,")?;
        code_output!(code, IDT2, "name: &'static str,")?;
        code_output!(code, IDT2, "stamp: u64,")?;
        code_output!(code, IDT2, "value: {},", self.get_data_type())?;
        code_output!(code, IDT1, "}\n")?;

        self.gen_signal_enum(code, msg)?;

        // start signal implementation
        code_output!(code, IDT1, "impl {}  {{", self.get_type_kamel())?;
        code_output!(code, IDT2, "pub fn new() -> Rc<RefCell<Box<dyn CanDbcSignal>>> {")?;
        code_output!(code, IDT3, "Rc::new(RefCell::new(Box::new({} {{", self.get_type_kamel())?;
        code_output!(code, IDT4, "status: CanDataStatus::Unset,")?;
        //code_output!(code, IDT4, "uid: DbcSignal::{},",)?;
        code_output!(code, IDT4, "name:\"{}\",", self.get_type_kamel())?;
        if self.size == 1 {
            code_output!(code, IDT4, "value: false,")?;
        } else {
            code_output!(code, IDT4, "value: 0_{},", self.get_data_type())?;
        }

        code_output!(code, IDT4, "stamp: 0,")?;
        code_output!(code, IDT4, "callback: None,")?;
        code_output!(code, IDT3, "})))")?;
        code_output!(code, IDT2, "}\n")?;

        code_output!(code, IDT2, "fn reset_value(&mut self) {")?;
        if self.size == 1 {
            code_output!(code, IDT3, "self.value= false;")?;
        } else {
            code_output!(code, IDT3, "self.value= 0_{};", self.get_data_type())?;
        }

        code_output!(code, IDT2, "}\n")?;

        if let Some(variants) = code.dbcfd.value_descriptions_for_signal(msg.id, self.name.as_str())
        {
            code_output!(
                code,
                IDT2,
                "pub fn get_as_def (&self) -> Dbc{} {{",
                self.get_type_kamel()
            )?;

            // float is not compatible with match
            if self.get_data_type() == "f64" {
                code_output!(
                    code,
                    IDT4,
                    "Dbc{}::_Other(self.get_typed_value())",
                    self.get_type_kamel()
                )?;
            } else {
                let mut count = 0;
                code_output!(code, IDT3, "match self.get_typed_value() {")?;
                for variant in variants {
                    if variant.a > self.max || variant.a < self.min {
                        code_output!(
                        code,
                        IDT4,
                        "// WARNING {} => Err(CanError::new(\"not-in-range\",\"({}) !!! {}({}) not in [{}..{}] range\")),",
                        variant.get_data_value(&self.get_data_type()),
                        variant.get_type_kamel(),
                        variant.a,
                        self.get_data_type(),
                        self.min,
                        self.max
                    )?;
                    } else {
                        count += 1;
                        code_output!(
                            code,
                            IDT4,
                            "{} => Dbc{}::{},",
                            variant.get_data_value(&self.get_data_type()),
                            self.get_type_kamel(),
                            variant.get_type_kamel()
                        )?;
                    }
                }

                // Help in buggy DBC file support
                if count != 2 || self.size != 1 {
                    code_output!(
                        code,
                        IDT4,
                        "_ => Dbc{}::_Other(self.get_typed_value()),",
                        self.get_type_kamel()
                    )?;
                }
                code_output!(code, IDT3, "}")?;
            }
            code_output!(code, IDT2, "}\n")?;

            code_output!(
                code,
                IDT2,
                "pub fn set_as_def (&mut self, signal_def: Dbc{}, data: &mut[u8])-> Result<(),CanError> {{",
                self.get_type_kamel()
            )?;

            code_output!(code, IDT3, "match signal_def {")?;
            for variant in variants {
                if variant.a > self.max || variant.a < self.min {
                    code_output!(
                        code,
                        IDT4,
                        "Dbc{}::{} => Err(CanError::new(\"not-in-range\",\"({}) !!! {}({}) not in [{}..{}] range\")),",
                        self.get_type_kamel(),
                        variant.get_type_kamel(),
                        variant.get_type_kamel(),
                        variant.a,
                        self.get_data_type(),
                        self.min,
                        self.max
                    )?;
                } else {
                    code_output!(
                        code,
                        IDT4,
                        "Dbc{}::{} => self.set_typed_value({}, data),",
                        self.get_type_kamel(),
                        variant.get_type_kamel(),
                        variant.get_data_value(&self.get_data_type())
                    )?;
                }
            }
            code_output!(
                code,
                IDT4,
                "Dbc{}::_Other(x) => self.set_typed_value(x,data)",
                self.get_type_kamel()
            )?;
            code_output!(code, IDT3, "}")?;
            code_output!(code, IDT2, "}")?;
        }

        // signal get typed_value
        code_output!(code, IDT2, "fn get_typed_value(&self) -> {} {{", self.get_data_type())?;
        code_output!(code, IDT3, "self.value")?;
        code_output!(code, IDT2, "}\n")?;

        // signal set_type_value
        code_output!(
            code,
            IDT2,
            "fn set_typed_value(&mut self, value:{}, data:&mut [u8]) -> Result<(),CanError> {{",
            self.get_data_type()
        )?;

        if self.size == 1 {
            code_output!(code, IDT3, "let value = value as u8;")?;
        } else if code.range_check && self.has_scaling() {
            code_output!(
                code,
                IDT3,
                "if value < {}_{} || {}_{} < value {{",
                self.min,
                self.get_data_type(),
                self.max,
                self.get_data_type()
            )?;
            code_output!(code,IDT4,
                    "return Err(CanError::new(\"invalid-signal-value\",format!(\"value={{}} not in [{}..{}]\",value)));", self.min, self.max)?;
            code_output!(code, IDT3, "}")?;
            code_output!(code, IDT3, "let factor = {}_f64;", self.factor)?;
            code_output!(code, IDT3, "let offset = {}_f64;", self.offset)?;
            code_output!(
                code,
                IDT3,
                "let value = ((value - offset) / factor) as {};",
                self.get_data_usize()
            )?;
        }

        if self.value_type == ValueType::Signed {
            code_output!(
                code,
                IDT3,
                "let value = {}::from_ne_bytes(value.to_ne_bytes());",
                self.get_data_usize()
            )?;
        }

        match self.byte_order {
            ByteOrder::LittleEndian => {
                let (start_bit, end_bit) = self.le_start_end_bit(msg)?;
                code_output!(
                    code,
                    IDT3,
                    "data.view_bits_mut::<Lsb0>()[{}..{}].store_le(value);",
                    start_bit,
                    end_bit
                )?;
            }
            ByteOrder::BigEndian => {
                let (start_bit, end_bit) = self.be_start_end_bit(msg)?;
                code_output!(
                    code,
                    IDT3,
                    "data.view_bits_mut::<Msb0>()[{}..{}].store_be(value);",
                    start_bit,
                    end_bit
                )?;
            }
        }

        code_output!(code, IDT3, "Ok(())")?;
        code_output!(code, IDT2, "}\n")?;

        // closing implementation
        code_output!(
            code,
            IDT1,
            "}} // {}::{} impl end\n",
            msg.get_type_kamel(),
            self.get_type_kamel()
        )?;

        Ok(())
    }

    fn gen_can_std_frame(&self, _code: &DbcCodeGen, _msg: &Message) -> io::Result<()> {
        Ok(())
    }

    fn gen_can_any_frame(&self, code: &DbcCodeGen, msg: &Message) -> io::Result<()> {
        match self.multiplexer_indicator {
            MultiplexIndicator::Plain => self.gen_can_std_frame(code, msg)?,
            MultiplexIndicator::Multiplexor
            | MultiplexIndicator::MultiplexedSignal(_)
            | MultiplexIndicator::MultiplexorAndMultiplexedSignal(_) => {
                // (optional) any shared handling for multiplexed cases
            }
}

        // fmt display for signal
        code_output!(code, IDT1, "impl fmt::Display for {} {{", self.get_type_kamel())?;
        code_output!(code, IDT2, "fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {")?;
        code_output!(
            code,
            IDT3,
            "let text=format!(\"{}:{{}}\", self.get_typed_value());",
            self.get_type_kamel()
        )?;
        code_output!(code, IDT3, "fmt.pad(&text)")?;
        code_output!(code, IDT2, "}")?;
        code_output!(code, IDT1, "}\n")?;

        // fmt debug for signal
        code_output!(code, IDT1, "impl fmt::Debug for {} {{", self.get_type_kamel())?;
        code_output!(
            code,
            IDT2,
            "fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {"
        )?;
        code_output!(code, IDT3, "format.debug_struct(\"{}\")", self.get_type_kamel())?;

        code_output!(code, IDT4, ".field(\"val\", &self.get_typed_value())")?;
        code_output!(code, IDT4, ".field(\"stamp\", &self.get_stamp())")?;
        code_output!(code, IDT4, ".field(\"status\", &self.get_status())")?;
        code_output!(code, IDT4, ".finish()")?;
        code_output!(code, IDT2, "}")?;
        code_output!(code, IDT1, "}\n")?;
        Ok(())
    }

    fn gen_code_signal(&self, code: &DbcCodeGen, msg: &Message) -> io::Result<()> {
        self.gen_signal_impl(code, msg)?;
        self.gen_can_any_frame(code, msg)?;
        self.gen_signal_trait(code, msg)?;
        Ok(())
    }
}

impl MsgCodeGen<&DbcCodeGen> for Message {
    fn gen_can_dbc_impl(&self, code: &DbcCodeGen) -> io::Result<()> {
        code_output!(code, IDT1, "pub struct DbcMessage {")?;
        code_output!(code, IDT2, "callback: Option<RefCell<Box<dyn CanMsgCtrl>>>,")?;
        code_output!(
            code,
            IDT2,
            "signals: [Rc<RefCell<Box<dyn CanDbcSignal>>>;{}],",
            self.signals.len()
        )?;
        code_output!(code, IDT2, "name: &'static str,")?;
        code_output!(code, IDT2, "status: CanBcmOpCode,")?;
        code_output!(code, IDT2, "listeners: i32,")?;
        code_output!(code, IDT2, "stamp: u64,")?;
        code_output!(code, IDT2, "id: u32,")?;
        code_output!(code, IDT1, "}\n")?;

        code_output!(code, IDT1, "impl DbcMessage {")?;

        // instantiate an empty message
        code_output!(code, IDT2, "pub fn new() -> Rc<RefCell<Box <dyn CanDbcMessage>>> {")?;
        code_output!(code, IDT3, "Rc::new(RefCell::new(Box::new (DbcMessage {")?;
        code_output!(code, IDT4, "id: {},", self.id.to_u32())?;
        code_output!(code, IDT4, "name: \"{}\",", self.get_type_kamel())?;
        code_output!(code, IDT4, "status: CanBcmOpCode::Unknown,")?;
        code_output!(code, IDT4, "listeners: 0,")?;
        code_output!(code, IDT4, "stamp: 0,")?;
        code_output!(code, IDT4, "callback: None,")?;
        code_output!(code, IDT4, "signals: [")?;
        for signal in &self.signals {
            code_output!(code, IDT5, "{}::new(),", signal.get_type_kamel())?;
        }
        code_output!(code, IDT4, "],")?;
        code_output!(code, IDT3, "})))")?;
        code_output!(code, IDT2, "}\n")?;

        // set all message signals values
        let args: Vec<String> = self.signals.iter()
            .map(|signal| format!("{}: {}", signal.get_type_snake(), signal.get_data_type()))
            .collect();

        code_output!(
            code,
            IDT2,
            "pub fn set_values(&mut self, {}, frame: &mut[u8]) -> Result<&mut Self, CanError> {{\n",
            args.join(", ")
        )?;

        for idx in 0..self.signals.len() {
            code_output!(
                code,
                IDT3,
                "match Rc::clone (&self.signals[{}]).try_borrow_mut() {{",
                idx
            )?;

            code_output!(
                code,
                IDT4,
                "Ok(mut signal) => signal.set_value(CanDbcType::{}({}), frame)?,",
                self.signals[idx].get_data_type().to_upper_camel_case(),
                self.signals[idx].get_type_snake()
            )?;
            code_output!(
                code,
                IDT4,
                "Err(_) => return Err(CanError::new(\"signal-set-values-fail\",\"Internal error {}:{}\")),",
                self.signals[idx].get_type_snake(),
                self.signals[idx].get_data_type().to_upper_camel_case()
            )?;

            code_output!(code, IDT3, "}")?;
        }
        code_output!(code, IDT3, "Ok(self)")?;
        code_output!(code, IDT2, "}")?;
        code_output!(code, IDT1, "}\n")?;

        Ok(())
    }

    fn gen_can_dbc_message(&self, code: &DbcCodeGen) -> io::Result<()> {
        // build message signal:type list
        code_output!(code, IDT1, "impl CanDbcMessage for DbcMessage {")?;
        code_output!(code, IDT2, "fn reset(&mut self) -> Result<(), CanError> {")?;
        code_output!(code, IDT3, "self.status=CanBcmOpCode::Unknown;")?;
        code_output!(code, IDT3, "self.stamp=0;")?;
        for idx in 0..self.signals.len() {
            code_output!(
                code,
                IDT3,
                "match Rc::clone (&self.signals[{}]).try_borrow_mut() {{",
                idx
            )?;

            code_output!(code, IDT4, "Ok(mut signal) => signal.reset(),",)?;
            code_output!(
                code,
                IDT4,
                "Err(_) => return Err(CanError::new(\"signal-reset-fail\",\"Internal error {}:{}\")),",
                self.signals[idx].get_type_snake(),
                self.signals[idx].get_data_type().to_upper_camel_case()
            )?;

            code_output!(code, IDT3, "}")?;
        }
        code_output!(code, IDT2, "Ok(())")?;
        code_output!(code, IDT1, "}\n")?;

        // update raw message value, then signals
        code_output!(
            code,
            IDT2,
            "fn update(&mut self, frame: &CanMsgData) -> Result<(), CanError> {"
        )?;
        code_output!(code, IDT3, "self.stamp= frame.stamp;")?;
        code_output!(code, IDT3, "self.status= frame.opcode;")?;
        code_output!(code, IDT3, "self.listeners= 0;")?;
        for idx in 0..self.signals.len() {
            code_output!(
                code,
                IDT3,
                "match Rc::clone (&self.signals[{}]).try_borrow_mut() {{",
                idx
            )?;

            code_output!(code, IDT4, "Ok(mut signal) => self.listeners += signal.update(frame),",)?;
            code_output!(
                code,
                IDT4,
                "Err(_) => return Err(CanError::new(\"signal-update-fail\",\"Internal error {}:{}\")),",
                self.signals[idx].get_type_snake(),
                self.signals[idx].get_data_type().to_upper_camel_case()
            )?;

            code_output!(code, IDT3, "}")?;
        }
        code_output!(code, IDT3, "match &self.callback {")?;
        code_output!(code, IDT4, "None => {},")?;
        code_output!(code, IDT4, "Some(callback) => {")?;
        code_output!(code, IDT5, "match callback.try_borrow() {")?;
        code_output!(
            code,
            IDT6,
            "Err(_) => println!(\"fail to get message callback reference\"),"
        )?;
        code_output!(code, IDT6, "Ok(cb_ref) => cb_ref.msg_notification(self),")?;
        code_output!(code, IDT5, "}")?;
        code_output!(code, IDT4, "}")?;
        code_output!(code, IDT3, "}")?;

        code_output!(code, IDT3, "Ok(())")?;
        code_output!(code, IDT2, "}\n")?;

        // get message signals collection
        code_output!(
            code,
            IDT2,
            "fn get_signals(&self) -> &[Rc<RefCell<Box<dyn CanDbcSignal>>>] {"
        )?;
        code_output!(code, IDT3, "&self.signals")?;
        code_output!(code, IDT2, "}\n")?;

        // get message active signals listeners
        code_output!(code, IDT2, "fn get_listeners(&self) -> i32 {")?;
        code_output!(code, IDT3, "self.listeners")?;
        code_output!(code, IDT2, "}\n")?;

        // set message notification callback
        code_output!(code, IDT2, "fn set_callback(&mut self, callback: Box<dyn CanMsgCtrl>)  {")?;
        code_output!(code, IDT3, "self.callback= Some(RefCell::new(callback));")?;
        code_output!(code, IDT2, "}\n")?;

        // get message name
        code_output!(code, IDT2, "fn get_name(&self) -> &'static str {")?;
        code_output!(code, IDT3, "self.name")?;
        code_output!(code, IDT2, "}\n")?;

        // get message status
        code_output!(code, IDT2, "fn get_status(&self) -> CanBcmOpCode {")?;
        code_output!(code, IDT3, "self.status")?;
        code_output!(code, IDT2, "}\n")?;

        // get message timestamp
        code_output!(code, IDT2, "fn get_stamp(&self) -> u64 {")?;
        code_output!(code, IDT3, "self.stamp")?;
        code_output!(code, IDT2, "}\n")?;

        // get message timestamp
        code_output!(code, IDT2, "fn get_id(&self) -> u32 {")?;
        code_output!(code, IDT3, "self.id")?;
        code_output!(code, IDT2, "}\n")?;

        // get message as_any
        code_output!(code, IDT2, "fn as_any(&mut self) -> &mut dyn Any {")?;
        code_output!(code, IDT3, "self")?;
        code_output!(code, IDT2, "}\n")?;

        code_output!(code, IDT1, "}} // end {} impl for CanDbcMessage", self.get_type_kamel())?;
        Ok(())
    }

    fn gen_code_message(&self, code: &DbcCodeGen) -> io::Result<()> {
        // message header
        code_output!(code, IDT0, "/// {} Message", self.name)?;
        code_output!(code, IDT0, "/// - ID: {0} (0x{0:x})", self.id.0)?;
        code_output!(code, IDT0, "/// - Size: {} bytes", self.size)?;
        if let Transmitter::NodeName(transmitter) = &self.transmitter {
            code_output!(code, IDT0, "/// - Transmitter: {}", transmitter)?;
        }
        if let Some(comment) = code.dbcfd.message_comment(self.id) {
            code_output!(code, IDT0, "///")?;
            for line in comment.trim().lines() {
                code_output!(code, IDT0, "/// {}", line)?;
            }
        }

        // per message module/name-space
        code_output!(code, IDT0, "pub mod {} {{ /// Message name space", self.get_type_kamel())?;
        code_output!(code, IDT1, "use sockcan::prelude::*;")?;
        code_output!(code, IDT1, "use bitvec::prelude::*;")?;
        code_output!(code, IDT1, "use std::any::Any;")?;
        code_output!(code, IDT1, "use std::cell::{RefCell};")?;
        code_output!(code, IDT1, "use std::rc::Rc;\n")?;
        code_output!(code, IDT1, "use std::fmt;\n")?;
        if code.serde_json {
            code_output!(code, IDT1, "use serde::{Deserialize, Serialize};")?;
        }

        // enumeration with all signal type
        code_output!(code, IDT1, "pub enum DbcSignal {")?;
        for signal in &self.signals {
            code_output!(code, IDT2, "{},", signal.get_type_kamel())?;
        }
        code_output!(code, IDT1, "}\n")?;

        // signals structures and implementation
        for signal in &self.signals {
            signal.gen_code_signal(code, self)?;
        }

        self.gen_can_dbc_impl(code)?;
        self.gen_can_dbc_message(code)?;

        code_output!(code, IDT0, "}} // end {} message\n", self.get_type_kamel())?;
        Ok(())
    }
}

pub trait Text2Str<T> {
    /// Write a line with indentation.
    ///
    /// # Errors
    /// Propagates any I/O error from the underlying writer.
    fn write(&self, indent: &str, text: T) -> io::Result<()>;
}

impl Text2Str<&str> for DbcCodeGen {
    fn write(&self, indent: &str, text: &str) -> io::Result<()> {
        let nl = "\n";
        if let Some(outfd) = &self.outfd {
            let mut outfd = outfd.try_clone()?;
            outfd.write_all(indent.as_bytes())?;
            outfd.write_all(text.as_bytes())?;
            outfd.write_all(nl.as_bytes())?;
        } else {
            let mut outfd = io::stdout();
            outfd.write_all(indent.as_bytes())?;
            outfd.write_all(text.as_bytes())?;
            outfd.write_all(nl.as_bytes())?;
        }

        Ok(())
    }
}

impl Text2Str<String> for DbcCodeGen {
    fn write(&self, indent: &str, text: String) -> io::Result<()> {
        Self::write(self, indent, text.as_str())
    }
}

impl DbcCodeGen {
    fn output<T>(&self, indent: &str, text: T) -> io::Result<()>
    where
        DbcCodeGen: Text2Str<T>,
    {
        Self::write(self, indent, text)
    }
}

impl DbcParser {
    #[must_use]
    pub fn new(uid: &'static str) -> Self {
        DbcParser {
            uid,
            range_check: true,
            serde_json: true,
            infile: None,
            outfile: None,
            header: None,
            whitelist: None,
            blacklist: None,
        }
    }

    pub fn dbcfile(&mut self, dbcfile: &str) -> &mut Self {
        self.infile = Some(dbcfile.to_owned());
        self
    }

    pub fn outfile(&mut self, outfile: &str) -> &mut Self {
        self.outfile = Some(outfile.to_owned());
        self
    }

    pub fn header(&mut self, header: &'static str) -> &mut Self {
        self.header = Some(header);
        self
    }

    pub fn whitelist(&mut self, canids: Vec<u32>) -> &mut Self {
        self.whitelist = Some(canids);
        self
    }

    pub fn blacklist(&mut self, canids: Vec<u32>) -> &mut Self {
        self.blacklist = Some(canids);
        self
    }

    pub fn range_check(&mut self, flag: bool) -> &mut Self {
        self.range_check = flag;
        self
    }

    pub fn serde_json(&mut self, flag: bool) -> &mut Self {
        self.serde_json = flag;
        self
    }

    fn check_list(canid: MessageId, list: &[u32]) -> bool {
        list.binary_search(&canid.0).is_ok()
    }
    /// Generate Rust code from the configured DBC.
    ///
    /// # Errors
    /// I/O errors reading the DBC or writing output; parsing errors.
    ///
    /// # Panics
    /// Panics if time formatting (`get_time("%c")`) fails.
    #[allow(clippy::too_many_lines)]
    pub fn generate(&mut self) -> io::Result<()> {
        let Some(infile) = &self.infile else {
            return Err(Error::other("setting dbcpath is mandatory"));
        };

        // open and parse dbc input file
        let mut dbcfd = match DbcObject::from_file(infile.as_str()) {
            Err(error) => return Err(Error::other(error.to_string())),
            Ok(dbcfd) => dbcfd,
        };

        // sort message by canid
        dbcfd.messages.sort_by(|a, b| a.id.0.cmp(&b.id.0));

        if let Some(mut list) = self.whitelist.clone() {
            list.sort_unstable();
            dbcfd.messages.retain(|msg| DbcParser::check_list(msg.id, &list));
        }

        if let Some(mut list) = self.blacklist.clone() {
            list.sort_unstable();
            dbcfd.messages.retain(|msg| !DbcParser::check_list(msg.id, &list));
        }

        // sort message by canid
        dbcfd.messages.sort_by(|a, b| a.id.0.cmp(&b.id.0));

        let outfd = match &self.outfile {
            Some(outfile) => {
                let outfd = File::create(outfile.as_str())?;
                Some(outfd)
            }
            None => None,
        };

        // open/create output file
        let code = DbcCodeGen {
            dbcfd,
            outfd,
            range_check: self.range_check,
            serde_json: self.serde_json,
        };

        match self.header {
            None => {}
            Some(header) => {
                code_output!(code, IDT1, header)?;
            }
        }

        // change Rust default to stick as much as possible on can names
        code_output!(
            code,
            IDT0,
            "// --------------------------------------------------------------",
        )?;
        code_output!(code, IDT0, "//       WARNING: Manual modification will be destroyed",)?;
        code_output!(
            code,
            IDT0,
            "// --------------------------------------------------------------",
        )?;
        code_output!(
            code,
            IDT0,
            "// - code generated from {} ({})",
            infile,
            get_time("%c").unwrap()
        )?;
        code_output!(code, IDT0, "// - update only with [dbc-parser|build.rs::DbcParser]",)?;
        code_output!(code, IDT0, "// - source code: https://github.com/redpesk-labs/canbus-rs",)?;
        code_output!(
            code,
            IDT0,
            "// - (C)IoT.bzh(2023), Author: Fulup Ar Foll, http://redpesk.bzh",
        )?;
        code_output!(code,IDT0,"// - License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$")?;
        code_output!(
            code,
            IDT0,
            "// -------------------------------------------------------------",
        )?;

        code_output!(code, IDT0, "mod {} {{", self.uid)?;
        code_output!(code, IDT0, "#![allow(non_upper_case_globals)]")?;
        code_output!(code, IDT0, "#![allow(non_camel_case_types)]")?;
        code_output!(code, IDT0, "#![allow(non_snake_case)]")?;
        code_output!(code, IDT0, "#![allow(dead_code)]")?;
        if code.serde_json {
            code_output!(code, IDT0, "extern crate serde;")?;
        }
        code_output!(code, IDT0, "extern crate bitvec;")?;
        code_output!(code, IDT0, "use sockcan::prelude::*;")?;
        code_output!(code, IDT0, "use std::cell::{RefCell,RefMut};")?;
        code_output!(code, IDT0, "use std::rc::{Rc};")?;
        code_output!(code, IDT0, "")?;

        // output messages/signals
        for message in &code.dbcfd.messages {
            message.gen_code_message(&code)?;
        }

        // enumeration with all signal type
        code_output!(code, IDT0, "enum DbcMessages {")?;
        for message in &code.dbcfd.messages {
            code_output!(code, IDT1, "{},", message.get_type_kamel())?;
        }
        code_output!(code, IDT0, "}\n")?;

        code_output!(code, IDT0, "pub struct CanMsgPool {")?;
        code_output!(code, IDT1, "uid: &'static str,")?;
        code_output!(
            code,
            IDT1,
            "pool: [Rc<RefCell<Box<dyn CanDbcMessage>>>;{}],",
            &code.dbcfd.messages.len()
        )?;
        code_output!(code, IDT0, "}\n")?;

        code_output!(code, IDT0, "impl CanMsgPool {")?;

        // extract canid from messages vector
        let canids: Vec<u32> =
            code.dbcfd.messages.iter().map(|msg| msg.id.to_u32()).collect();

        code_output!(code, IDT1, "pub fn new(uid: &'static str) -> Self {")?;
        code_output!(code, IDT2, "CanMsgPool {")?;
        code_output!(code, IDT3, "uid: uid,")?;
        code_output!(code, IDT3, "pool: [")?;
        for idx in 0..code.dbcfd.messages.len() {
            code_output!(
                code,
                IDT4,
                "{}::DbcMessage::new(),",
                code.dbcfd.messages[idx].get_type_kamel()
            )?;
        }
        code_output!(code, IDT3, "]")?;
        code_output!(code, IDT2, "}")?;
        code_output!(code, IDT1, "}")?;
        code_output!(code, IDT0, "}\n")?;

        code_output!(code, IDT0, "impl CanDbcPool for CanMsgPool {")?;
        code_output!(
            code,
            IDT1,
            "fn get_messages(&self) -> &[Rc<RefCell<Box<dyn CanDbcMessage>>>] {"
        )?;

        code_output!(code, IDT2, "&self.pool")?;
        code_output!(code, IDT1, "}\n")?;
        code_output!(code, IDT1, "fn get_ids(&self) -> &[u32] {")?;
        code_output!(code, IDT2, "&{:?}", canids)?;
        code_output!(code, IDT1, "}\n")?;

        code_output!(
            code,
            IDT1,
            "fn get_mut(&self, canid: u32) -> Result<RefMut<'_, Box<dyn CanDbcMessage>>, CanError> {"
        )?;
        code_output!(
            code,
            IDT2,
            "let search= self.pool.binary_search_by(|msg| msg.borrow().get_id().cmp(&canid));",
        )?;
        code_output!(code, IDT2, "match search {")?;
        code_output!(code, IDT3, "Ok(idx) => {")?;
        code_output!(code, IDT4, "match self.pool[idx].try_borrow_mut() {")?;
        code_output!(
            code,
            IDT5,
            "Err(_code) => Err(CanError::new(\"message-get_mut\", \"internal msg pool error\")),"
        )?;
        code_output!(code, IDT5, "Ok(mut_ref) => Ok(mut_ref),")?;
        code_output!(code, IDT4, "}")?;
        code_output!(code, IDT3, "},")?;
        code_output!(code,IDT3,"Err(_) => Err(CanError::new(\"fail-canid-search\", format!(\"canid:{} not found\",canid))),")?;
        code_output!(code, IDT2, "}")?;
        code_output!(code, IDT1, "}\n")?;

        code_output!(
            code,
            IDT1,
            "fn update(&self, data: &CanMsgData) -> Result<RefMut<'_, Box<dyn CanDbcMessage>>, CanError> {"
        )?;
        code_output!(code, IDT2, "let mut msg= match self.get_mut(data.canid) {")?;
        code_output!(code, IDT3, "Err(error) => return Err(error),")?;
        code_output!(code, IDT3, "Ok(msg_ref) => msg_ref,")?;
        code_output!(code, IDT2, "};")?;
        code_output!(code, IDT2, "msg.update(data)?;")?;
        code_output!(code, IDT2, "Ok(msg)")?;
        code_output!(code, IDT1, "}")?;

        code_output!(code, IDT0, " }")?;
        code_output!(code, IDT0, "} // end dbc generated parser")?;
        Ok(())
    }
}
