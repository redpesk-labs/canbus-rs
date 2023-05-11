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
 *
*/

use super::cglue;
use crate::prelude::*;
use std::mem::{self, MaybeUninit};

pub struct SockJ1939Filter {
    filter: Vec<cglue::j1939_filter>,
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanJ1939OpCode {
    RxRead,
    RxError,
}

pub struct SockJ1939Msg {
    pub opcode: CanJ1939OpCode,
    pub info: CanRecvInfo,
    pub frame: Vec<u8>,
}

impl SockJ1939Msg {
    pub fn get_iface(&self) -> i32 {
        self.info.iface
    }

    pub fn get_opcode(&self) -> CanJ1939OpCode {
        self.opcode.clone()
    }

    pub fn get_stamp(&self) -> u64 {
        self.info.stamp
    }

    pub fn get_dest(&self) -> CanJ1939Header {
        unsafe { self.info.proto.j1939.dst }
    }

    pub fn get_pgn(&self) -> u32 {
        unsafe { self.info.proto.j1939.pgn }
    }

    pub fn get_len(&self) -> usize {
        self.frame.len()
    }

    pub fn get_data(&self) -> &[u8] {
        &self.frame
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
        };

        let iface = SockCanHandle::map_can_iface(sockfd, candev);
        if iface < 0 {
            return Err(CanError::new("fail-socketcan-iface", cglue::get_perror()));
        }

        #[allow(invalid_value)]
        let mut canaddr: cglue::sockaddr_can = unsafe { MaybeUninit::uninit().assume_init() };
        canaddr.can_family = cglue::can_SOCK_x_AF_CAN as u16;
        canaddr.can_ifindex = iface;

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
        let mut buffer:Vec<u8>= Vec::with_capacity(cglue::can_J1939_x_MAX_TP_PACKET_SIZE as usize);
        let info = self.recv_can_msg(buffer.as_mut_ptr(), cglue::can_J1939_x_MAX_TP_PACKET_SIZE as u32);

        if info.count > 0 {
            unsafe {buffer.set_len(info.count as usize)};
            SockJ1939Msg {
                info: info,
                opcode: CanJ1939OpCode::RxRead,
                frame: buffer,
            }
        } else {
            SockJ1939Msg {
                info: info,
                opcode: CanJ1939OpCode::RxError,
                frame: Vec::new(),
            }
        }
    }
}

impl SockJ1939Filter {
    pub fn new() -> Self {
        // J1939 filter have at least one filter
        SockJ1939Filter { filter: Vec::new() }
    }

    pub fn add_pgn(&mut self, pgn: u32) -> &mut Self {
        let mut filter= unsafe { mem::zeroed::<cglue::j1939_filter>() };
        filter.pgn= pgn;
        filter.pgn_mask= !0;
        self.filter.push(filter);
        self
    }

    pub fn add_name(&mut self, name: u64) -> &mut Self {
        let mut filter= unsafe { mem::zeroed::<cglue::j1939_filter>() };
        filter.name=name;
        filter.name_mask=!0;
        self.filter.push(filter);
        self
    }

    pub fn apply(&self, sock: &SockCanHandle) -> Result<(), CanError> {
        let filter_len = self.filter.len();
        let j1939_filter = self.filter.as_slice();

        if filter_len > cglue::can_J1939_x_FILTER_MAX as usize {
            return Err(CanError::new(
                "j1939-filter-number",
                "to many j1939 filters",
            ));
        }

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
