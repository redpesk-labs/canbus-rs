/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 * References:
 *    https://www.kernel.org/doc/html/v5.7/networking/j1939.html
 *    https://www.kvaser.com/about-can/higher-layer-protocols/j1939-introduction/
 *    https://gitlab.cern.ch/atlas-dcs-common-software/socketcan-utils/blob/8839679a4cf19a0c44c207b61683b2a707779a6e/can-j1939-kickstart.md
 *    https://github.com/linux-can/can-utils/blob/master/testj1939.c
 *    https://www.engr.colostate.edu/~jdaily/J1939/candata.html
 *    https://www.csselectronics.com/pages/nmea-2000-n2k-intro-tutorial#fast-packet
 *
*/

use super::cglue;
use crate::prelude::*;
use std::cell::{RefCell, RefMut};
use std::mem::{self, MaybeUninit};

const _MAX_N2K_FAST_SZ: usize = 223; // Max N2K data with 32 packets
const _MAX_N2K_PACK_SZ: usize = 8; // Individual packet are 8 bytes
const MAX_J1939_PKG_SZ: usize = cglue::can_J1939_x_MAX_TP_PACKET_SIZE as usize;

pub struct SockJ1939Ecu {
    name: u64,
    pgn: u32,
    addr: u8,
}

impl SockJ1939Ecu {
    pub fn new(name: u64) -> Self {
        SockJ1939Ecu {
            name: name,
            pgn: cglue::can_J1939_x_NO_PGN,
            addr: cglue::can_J1939_x_NO_ADDR as u8,
        }
    }

    pub fn set_addr(&mut self, addr: u8) -> &mut Self {
        self.addr = addr;
        self
    }

    pub fn set_pgn(&mut self, pgn: u32) -> &mut Self {
        self.pgn = pgn;
        self
    }
}

pub enum SockJ1939Addr {
    Filter(SockJ1939Ecu),
    Promiscuous,
}

pub struct SockJ1939Msg {
    pub opcode: SockCanOpCode,
    pub info: CanRecvInfo,
}

impl SockJ1939Msg {
    pub fn get_iface(&self) -> i32 {
        self.info.iface
    }

    pub fn get_opcode(&self) -> SockCanOpCode {
        self.opcode.clone()
    }

    pub fn get_stamp(&self) -> u64 {
        self.info.stamp
    }

    pub fn get_info(&self) -> Result<CanJ1939Info, CanError> {
        match self.info.proto {
            CanProtoInfo::J1939(info) => Ok(info),
            _ => Err(CanError::new(
                "sockj1939-msg-invalid",
                "No J1939 info within this message",
            )),
        }
    }

    pub fn get_pgn(&self) -> u32 {
        match self.info.proto {
            CanProtoInfo::J1939(info) => info.pgn,
            _ => 0,
        }
    }

    pub fn get_len(&self) -> usize {
        match &self.opcode {
            SockCanOpCode::RxRead(data) => data.len(),
            _ => 0,
        }
    }

    pub fn get_data<'a>(&'a self) -> &'a [u8] {
        match &self.opcode {
            SockCanOpCode::RxRead(data) => data.as_slice(),
            _ => &[0; 0],
        }
    }
}

pub trait SockCanJ1939 {
    fn open_j1939<T>(
        candev: T,
        mode: SockJ1939Addr,
        timestamp: CanTimeStamp,
    ) -> Result<SockCanHandle, CanError>
    where
        SockCanHandle: CanIFaceFrom<T>;

    fn get_j1939_frame(&self) -> SockJ1939Msg;
}

impl SockCanJ1939 for SockCanHandle {
    fn open_j1939<T>(
        candev: T,
        mode: SockJ1939Addr,
        timestamp: CanTimeStamp,
    ) -> Result<SockCanHandle, CanError>
    where
        SockCanHandle: CanIFaceFrom<T>,
    {
        let sockfd = unsafe {
            cglue::socket(
                cglue::can_SOCK_x_PF_CAN as i32,
                cglue::can_SOCK_x_DGRAM as i32,
                cglue::can_SOCK_x_J1939 as i32,
            )
        };
        if sockfd < 0 {
            return Err(CanError::new("fail-socketcan-open", cglue::get_perror()));
        }

        let mut sockcan = SockCanHandle {
            sockfd: sockfd,
            mode: SockCanMod::J1939,
            callback: None,
        };

        let iface = SockCanHandle::map_can_iface(sockfd, candev);
        if iface < 0 {
            return Err(CanError::new("fail-socketcan-iface", cglue::get_perror()));
        }

        #[allow(invalid_value)]
        let mut canaddr: cglue::sockaddr_can = unsafe { MaybeUninit::uninit().assume_init() };
        canaddr.can_family = cglue::can_SOCK_x_AF_CAN as u16;
        canaddr.can_ifframe_idx = iface;

        match mode {
            SockJ1939Addr::Promiscuous => {
                let flag: u32 = 1;
                let status = unsafe {
                    cglue::setsockopt(
                        sockfd,
                        cglue::can_J1939_x_SOL_CAN_J1939 as i32,
                        cglue::can_J1939_x_SO_PROMISC as i32,
                        &flag as *const _ as *const std::ffi::c_void,
                        mem::size_of::<u32>() as cglue::socklen_t,
                    )
                };

                if status < 0 {
                    return Err(CanError::new(
                        "fail-sockj1939-promiscuous",
                        cglue::get_perror(),
                    ));
                }

                canaddr.can_addr.j1939 = cglue::sockaddr_can__bindgen_ty_1__bindgen_ty_2 {
                    name: cglue::can_J1939_x_NO_NAME as u64,
                    pgn: cglue::can_J1939_x_NO_PGN,
                    addr: cglue::can_J1939_x_NO_ADDR as u8,
                };
            }

            SockJ1939Addr::Filter(ecu) => {
                if ecu.addr == cglue::can_J1939_x_NO_ADDR as u8 {
                    // broadcast require for address claim
                    let flag: u32 = 1;
                    let status = unsafe {
                        cglue::setsockopt(
                            sockfd,
                            cglue::can_SOCK_x_SOL_SOCKET as i32,
                            cglue::can_SOCK_x_SO_BROADCAST as i32,
                            &flag as *const _ as *const std::ffi::c_void,
                            mem::size_of::<u32>() as cglue::socklen_t,
                        )
                    };

                    if status < 0 {
                        return Err(CanError::new(
                            "fail-sockj1939-broadcast",
                            cglue::get_perror(),
                        ));
                    }
                }

                canaddr.can_addr.j1939 = cglue::sockaddr_can__bindgen_ty_1__bindgen_ty_2 {
                    name: ecu.name,
                    pgn: ecu.pgn,
                    addr: cglue::can_J1939_x_IDLE_ADDR as u8,
                };
            }
        }

        let sockaddr = cglue::__CONST_SOCKADDR_ARG {
            __sockaddr__: &canaddr as *const _ as *const cglue::sockaddr,
        };

        let status = unsafe {
            cglue::bind(
                sockfd,
                sockaddr,
                mem::size_of::<cglue::sockaddr_can>() as cglue::socklen_t,
            )
        };
        if status < 0 {
            return Err(CanError::new("fail-sockj1939-bind", cglue::get_perror()));
        }

        match sockcan.set_timestamp(timestamp) {
            Err(error) => return Err(error),
            Ok(_value) => {}
        }

        Ok(sockcan)
    }

    fn get_j1939_frame(&self) -> SockJ1939Msg {
        #[allow(invalid_value)]
        let mut buffer: [u8; MAX_J1939_PKG_SZ] = unsafe { MaybeUninit::uninit().assume_init() };

        // read raw frame from canbus
        let info = self.get_raw_frame(&mut buffer);
        if info.count < 0 {
            return SockJ1939Msg {
                info: info,
                opcode: SockCanOpCode::RxError(CanError::new(
                    "j1939-read-fail",
                    "fail to read message from canbus",
                )),
            };
        }

        // if fast-packet or any other user land protocol is register let's call it now
        if let Some(callback) = &self.callback {
            let opcode = match callback.try_borrow() {
                Err(_) => SockCanOpCode::RxError(CanError::new(
                    "can-recv-callback",
                    "(internal) Fail to fet ref_mut".to_string(),
                )),
                Ok(callback) => callback.check_frame(&buffer, &info),
            };
            SockJ1939Msg {
                info: info,
                opcode: opcode,
            }
        } else {
            // in j1939 message len can be anything from 1 to
            let mut data = buffer.to_vec();
            data.truncate(info.count as usize);
            SockJ1939Msg {
                info: info,
                opcode: SockCanOpCode::RxRead(data),
            }
        }
    }
}

pub struct SockJ1939Fast {
    pgn: u32,        // msg PGN
    frame_idx: u8,   // current chunk index
    frame_count: u8, // current frame frame_count number
    frame_len: u16,  // data to be read
    data_idx: usize, // data index
    data: Vec<u8>,   // data buffer
}

impl SockJ1939Fast {
    pub fn new(pgn: u32, dbc_len: i32) -> Self {
        let capacity = if dbc_len < 0 {
            MAX_J1939_PKG_SZ
        } else {
            dbc_len as usize
        };

        SockJ1939Fast {
            pgn: pgn,
            frame_idx: 0,
            frame_count: 0,
            frame_len: 0,
            data_idx: 0,
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn reset(&mut self) {
        self.frame_idx = 0;
        self.data_idx = 0;
    }

    pub fn push(&mut self, buffer: &[u8]) -> SockCanOpCode {
        println!("buffer: {:#04x}:{:#04x}", buffer[0], buffer[1]);

        // first message has no frame_idx but len on 2 bytes
        if self.frame_idx == 0 {
            // numero serial number is coded on initial 4 bits of the message
            self.frame_count = buffer[0] >> 4;

            // check packet len fit PGN description
            self.frame_len = ((buffer[0] & 0x0f) as u16 * 256 + buffer[1] as u16);
            if (self.frame_len as usize > self.data.len()){
                return SockCanOpCode::RxError(CanError::new(
                    "j1939-fastpkg-pgnlen",
                    format!("message pgn:{} len:{} bigger than capacity: {}", self.pgn, self.frame_len, self.data.len()),
                ));
            }

            for idx in 2..8 {
                self.data[self.data_idx] = buffer[idx];
                self.data_idx += 1;
            }
            self.frame_idx = 1;
        } else {
            if (self.frame_idx != (buffer[0] & 0x0f)) || self.frame_count != (buffer[0] >> 4) {
                self.reset();
                return SockCanOpCode::RxError(CanError::new(
                    "j1939-fastpkg-sequence",
                    "message sequence ordering broken",
                ));
            }

            for idx in 1..8 {
                self.data[self.size] = buffer[idx];
                self.size += 1;
                if self.size == self.data.len() {
                    self.reset();
                    return SockCanOpCode::RxRead(self.data.clone());
                }
            }
            self.frame_idx += 1;
        };
        SockCanOpCode::RxPartial(self.frame_idx)
    }
}

pub struct SockJ1939Filters {
    filter: Vec<cglue::j1939_filter>,
    fastpkg: Vec<RefCell<SockJ1939Fast>>,
}

// NMEA2000 does not use J1939 TP mechanism but in place have a custom FastPacket protocol
// Reference: https://canboat.github.io/canboat/canboat.html#pgn-126976
// this protocol uses standard 8 byte frames, with following data usage
// 1st packet: DATA[8]= LEN[2]+DATA[6] 2nd,... DATA[8]=IDX[1],DATA[7]  (max 32 packets)
// note: packet should be read in sequence or read fail
impl SockCanCtrl for SockJ1939Filters {
    fn check_frame(&self, data: &[u8], recv: &CanRecvInfo) -> SockCanOpCode {
        let info = match recv.proto {
            CanProtoInfo::J1939(info) => info,
            _ => return SockCanOpCode::RxInvalid,
        };

        match self.search_pgn(info.pgn) {
            // if fast packet process partial data, else return msg data as it
            Err(_error) => SockCanOpCode::RxRead(data.to_vec()),
            Ok(mut fast) => fast.push(data),
        }
    }
}

impl SockJ1939Filters {
    pub fn new() -> Self {
        // J1939 filter have at least one filter
        SockJ1939Filters {
            filter: Vec::new(),
            fastpkg: Vec::new(),
        }
    }

    pub fn get_fastlen(&self) -> usize {
        self.fastpkg.len()
    }

    pub fn add_pgn(&mut self, pgn: u32) -> &mut Self {
        let mut filter = unsafe { mem::zeroed::<cglue::j1939_filter>() };
        filter.pgn = pgn;
        filter.pgn_mask = !0;
        self.filter.push(filter);
        self
    }

    pub fn add_fast(&mut self, pgn: u32, len: usize) -> &mut Self {
        self.fastpkg
            .push(RefCell::new(SockJ1939Fast::new(pgn, len)));
        self.add_pgn(pgn);
        self
    }

    pub fn search_pgn(&self, pgn: u32) -> Result<RefMut<SockJ1939Fast>, CanError> {
        let search = self
            .fastpkg
            .binary_search_by(|pkg| pkg.borrow().pgn.cmp(&pgn));
        match search {
            Ok(idx) => match self.fastpkg[idx].try_borrow_mut() {
                Err(_code) => Err(CanError::new(
                    "message-get_mut",
                    "internal fastpkg pool error",
                )),
                Ok(mut_ref) => Ok(mut_ref),
            },
            Err(_) => Err(CanError::new(
                "fail-fastpgn-search",
                format!("canid:{} not found", pgn),
            )),
        }
    }

    pub fn add_name(&mut self, name: u64) -> &mut Self {
        let mut filter = unsafe { mem::zeroed::<cglue::j1939_filter>() };
        filter.name = name;
        filter.name_mask = !0;
        self.filter.push(filter);
        self
    }

    pub fn apply(&mut self, sock: &SockCanHandle) -> Result<(), CanError> {
        // sort fast packet vector list
        self.fastpkg
            .sort_by(|a, b| a.borrow().pgn.cmp(&b.borrow().pgn));

        // build filter list
        let filter_len = self.filter.len();
        let j1939_filter = self.filter.as_slice();
        if filter_len > cglue::can_J1939_x_FILTER_MAX as usize {
            return Err(CanError::new(
                "j1939-filter-number",
                "to many j1939 filters",
            ));
        }

        // register filter list
        let status = unsafe {
            cglue::setsockopt(
                sock.sockfd,
                cglue::can_J1939_x_SOL_CAN_J1939 as i32,
                cglue::can_J1939_x_SO_FILTER as i32,
                j1939_filter as *const _ as *const std::ffi::c_void,
                (mem::size_of::<cglue::j1939_filter>() * filter_len) as cglue::socklen_t,
            )
        };

        if status < 0 {
            return Err(CanError::new("fail-j1939-filter", cglue::get_perror()));
        } else {
            Ok(())
        }
    }
}
