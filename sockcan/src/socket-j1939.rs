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
use log::{debug, warn};

use super::cglue;
use crate::prelude::*;
use std::cell::{RefCell, RefMut};
use std::mem::{self};

const MAX_N2K_FAST_SZ: u16 = 223; // Max N2K data with 32 packets
const MAX_N2K_PACK_SZ: isize = 8; // Individual packet are 8 bytes
const MAX_J1939_PKG_SZ: u32 = cglue::can_J1939_x_MAX_TP_PACKET_SIZE;

pub struct SockJ1939Ecu {
    name: u64,
    pgn: u32,
    addr: u8,
}

impl SockJ1939Ecu {
    #[must_use]
    pub fn new(name: u64) -> Self {
        SockJ1939Ecu {
            name,
            pgn: cglue::can_J1939_x_NO_PGN,
            addr: u8::try_from(cglue::can_J1939_x_NO_ADDR).unwrap_or(u8::MAX),
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
    #[must_use]
    pub fn get_iface(&self) -> i32 {
        self.info.iface
    }

    #[must_use]
    pub fn get_opcode(&self) -> SockCanOpCode {
        self.opcode.clone()
    }

    #[must_use]
    pub fn get_stamp(&self) -> u64 {
        self.info.stamp
    }
    /// Extracts high-level J1939 metadata (e.g., source address, PGN, priority,
    /// and timestamp) from this received message.
    ///
    /// # Returns
    /// A `CanJ1939Info` structure containing decoded header fields and any
    /// associated metadata.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the underlying buffer is shorter than the minimum J1939 header size;
    /// - the PGN or address fields cannot be decoded (e.g., invalid bit layout);
    /// - integer conversions overflow/underflow or violate expected ranges;
    /// - required timestamp or ancillary data is missing or inconsistent.
    pub fn get_info(&self) -> Result<CanJ1939Info, CanError> {
        match self.info.proto {
            CanProtoInfo::J1939(info) => Ok(info),
            _ => Err(CanError::new("sockj1939-msg-invalid", "No J1939 info within this message")),
        }
    }

    #[must_use]
    pub fn get_pgn(&self) -> u32 {
        match self.info.proto {
            CanProtoInfo::J1939(info) => info.pgn,
            _ => 0,
        }
    }

    #[must_use]
    pub fn get_len(&self) -> usize {
        match &self.opcode {
            SockCanOpCode::RxRead(data) => data.len(),
            _ => 0,
        }
    }

    #[must_use]
    pub fn get_data(&self) -> &[u8] {
        match &self.opcode {
            SockCanOpCode::RxRead(data) => data.as_slice(),
            _ => &[],
        }
    }
}

pub trait SockCanJ1939 {
    /// Opens a J1939 socket on the given CAN interface with the requested addressing mode
    /// and timestamp configuration.
    ///
    /// # Parameters
    /// - `candev`: Interface identifier (e.g., `"can0"` or an index) convertible via `CanIFaceFrom<T>`.
    /// - `mode`: J1939 addressing mode (e.g., ECU/broadcast/unicast parameters).
    /// - `timestamp`: Desired timestamping mode for received frames.
    ///
    /// # Returns
    /// A configured `SockCanHandle` ready for J1939 operations.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the socket cannot be created (e.g., `socket()` fails or the protocol is unsupported);
    /// - binding the socket to the interface/address fails (invalid interface, permission error, or OS-level failure);
    /// - requested options (e.g., timestamping, broadcast/promiscuous flags) cannot be applied (`setsockopt` failure);
    /// - conversions/validations of lengths, pointers, or numeric casts fail;
    /// - `candev` cannot be converted by the `CanIFaceFrom<T>` implementation.
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
        // Safer: fall back to i32::MAX if the constant doesn't fit
        let pf_can = i32::try_from(cglue::can_SOCK_x_PF_CAN).unwrap_or(i32::MAX);
        let dgram = i32::try_from(cglue::can_SOCK_x_DGRAM).unwrap_or(i32::MAX);
        let j1939 = i32::try_from(cglue::can_SOCK_x_J1939).unwrap_or(i32::MAX);
        let sockfd = unsafe { cglue::socket(pf_can, dgram, j1939) };

        if sockfd < 0 {
            return Err(CanError::new("fail-socketcan-open", cglue::get_perror()));
        }

        let mut sockcan = SockCanHandle { sockfd, mode: SockCanMod::J1939, callback: None };

        let iface = SockCanHandle::map_can_iface(sockfd, candev);
        if iface < 0 {
            return Err(CanError::new("fail-socketcan-iface", cglue::get_perror()));
        }

        #[allow(invalid_value)]
        let mut canaddr: cglue::sockaddr_can = unsafe { std::mem::zeroed() };
        canaddr.can_family = u16::try_from(cglue::can_SOCK_x_AF_CAN).unwrap_or(u16::MAX);
        canaddr.can_ifindex = iface;

        match mode {
            SockJ1939Addr::Promiscuous => {
                let flag: u32 = 1;
                let sol_can_j1939 =
                    i32::try_from(cglue::can_J1939_x_SOL_CAN_J1939).unwrap_or(i32::MAX);
                let so_promisc = i32::try_from(cglue::can_J1939_x_SO_PROMISC).unwrap_or(i32::MAX);
                let optlen =
                    cglue::socklen_t::try_from(core::mem::size_of::<u32>()).unwrap_or(u32::MAX);
                let status = unsafe {
                    cglue::setsockopt(
                        sockfd,
                        sol_can_j1939,
                        so_promisc,
                        (&raw const flag).cast::<std::ffi::c_void>(),
                        optlen,
                    )
                };

                if status < 0 {
                    return Err(CanError::new("fail-sockj1939-promiscuous", cglue::get_perror()));
                }

                canaddr.can_addr.j1939 = cglue::sockaddr_can__bindgen_ty_1__bindgen_ty_2 {
                    name: u64::from(cglue::can_J1939_x_NO_NAME),
                    pgn: cglue::can_J1939_x_NO_PGN,
                    addr: u8::try_from(cglue::can_J1939_x_NO_ADDR).unwrap_or(u8::MAX),
                };
            },

            SockJ1939Addr::Filter(ecu) => {
                if ecu.addr == u8::try_from(cglue::can_J1939_x_NO_ADDR).unwrap_or(u8::MAX) {
                    // broadcast require for address claim
                    let flag: u32 = 1;
                    let sol_socket =
                        i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX);
                    let so_broadcast =
                        i32::try_from(cglue::can_SOCK_x_SO_BROADCAST).unwrap_or(i32::MAX);
                    let optlen =
                        cglue::socklen_t::try_from(core::mem::size_of::<u32>()).unwrap_or(u32::MAX);
                    let status = unsafe {
                        cglue::setsockopt(
                            sockfd,
                            sol_socket,
                            so_broadcast,
                            (&raw const flag).cast::<std::ffi::c_void>(),
                            optlen,
                        )
                    };

                    if status < 0 {
                        return Err(CanError::new("fail-sockj1939-broadcast", cglue::get_perror()));
                    }
                }

                canaddr.can_addr.j1939 = cglue::sockaddr_can__bindgen_ty_1__bindgen_ty_2 {
                    name: ecu.name,
                    pgn: ecu.pgn,
                    addr: u8::try_from(cglue::can_J1939_x_IDLE_ADDR).unwrap_or(u8::MAX),
                };
            },
        }

        let sockaddr = cglue::__CONST_SOCKADDR_ARG {
            __sockaddr__: (&raw const canaddr).cast::<cglue::sockaddr>(),
        };

        let namelen = cglue::socklen_t::try_from(core::mem::size_of::<cglue::sockaddr_can>())
            .unwrap_or(u32::MAX);
        let status = unsafe { cglue::bind(sockfd, sockaddr, namelen) };
        if status < 0 {
            return Err(CanError::new("fail-sockj1939-bind", cglue::get_perror()));
        }

        match sockcan.set_timestamp(timestamp) {
            Err(error) => return Err(error),
            Ok(_value) => {},
        }

        Ok(sockcan)
    }

    fn get_j1939_frame(&self) -> SockJ1939Msg {
        // Safe zero-init: avoids UB and the clippy::uninit_assumed_init lint
        let mut buffer: [u8; MAX_J1939_PKG_SZ as usize] = [0u8; MAX_J1939_PKG_SZ as usize];

        // read raw frame from canbus
        let info = self.get_raw_frame(&mut buffer);
        if info.count < 0 {
            return SockJ1939Msg {
                info,
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
            SockJ1939Msg { opcode, info }
        } else {
            // in j1939 message len can be anything from 1 to
            let mut data = buffer.to_vec();
            let n = usize::try_from(info.count).unwrap_or(0);
            data.truncate(n);
            SockJ1939Msg { info, opcode: SockCanOpCode::RxRead(data) }
        }
    }
}

pub struct SockJ1939Fast {
    pgn: u32,        // msg PGN
    frame_idx: u8,   // current chunk index
    frame_count: u8, // current frame frame_count number
    frame_len: u16,  // data to be read
    data: Vec<u8>,   // data buffer
    capacity: u16,   // data maximum capacity
}

impl SockJ1939Fast {
    #[must_use]
    pub fn new(pgn: u32, dbc_len: u16, mut capacity: u16) -> Self {
        if capacity == 0 {
            capacity = dbc_len;
        } else if u32::from(capacity) > MAX_J1939_PKG_SZ {
            capacity = MAX_N2K_FAST_SZ;
        }

        SockJ1939Fast {
            pgn,
            frame_idx: 0,
            frame_count: 0,
            frame_len: 0,
            capacity,
            data: Vec::with_capacity(capacity as usize),
        }
    }

    pub fn reset(&mut self) {
        self.frame_idx = 0;
        self.data.clear();
    }

    pub fn push(&mut self, buffer: &[u8], _len: isize) -> SockCanOpCode {
        //println!("buffer: {:#02x?}:{:#02x?}  len:{}", buffer[0], buffer[1],len);
        #[cfg(debug_assertions)]
        debug!("buffer: {:#02x?}:{:#02x?}  len:{}", buffer[0], buffer[1], _len);

        // first message has no frame_idx but len on 2 bytes
        if buffer[0].trailing_zeros() >= 4 {
            // numero serial number is coded on initial 4 bits of the message
            self.frame_count = buffer[0] >> 4;

            // previous message we uncompleted
            if !self.data.is_empty() {
                warn!("data lost: frame_len:{} data_len:{}", self.frame_len, self.data.len());
                self.reset();
            }

            // check packet len fit PGN description
            self.frame_len = u16::from(buffer[1]);
            if self.frame_len > self.capacity {
                return SockCanOpCode::RxError(CanError::new(
                    "j1939-fastpkg-pgnlen",
                    format!(
                        "message pgn:{} len:{} bigger than capacity: {}",
                        self.pgn,
                        self.frame_len,
                        self.data.len()
                    ),
                ));
            }

            self.data.extend_from_slice(&buffer[2..8]);
            self.frame_idx = 1;
        } else {
            if (self.frame_idx != (buffer[0] & 0x0f)) || self.frame_count != (buffer[0] >> 4) {
                self.reset();
                return SockCanOpCode::RxError(CanError::new(
                    "j1939-fastpkg-sequence",
                    format!("pgn:{} message sequence ordering broken", self.pgn),
                ));
            }

            for &b in buffer.iter().skip(1).take(7) {
                self.data.push(b);
                if self.frame_len as usize == self.data.len() {
                    let response = SockCanOpCode::RxRead(self.data.clone());
                    self.reset();
                    return response;
                }
            }
            self.frame_idx += 1;
        }
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
        let CanProtoInfo::J1939(info) = recv.proto else {
            return SockCanOpCode::RxInvalid;
        };

        // fast packet should be 8 bytes long
        if recv.count > MAX_N2K_PACK_SZ {
            return SockCanOpCode::RxInvalid;
        }

        match self.search_pgn(info.pgn) {
            // if fast packet process partial data, else return msg data as it
            Err(_error) => SockCanOpCode::RxRead(data.to_vec()),
            Ok(mut fast) => fast.push(data, recv.count),
        }
    }
}

impl Default for SockJ1939Filters {
    fn default() -> Self {
        Self::new()
    }
}

impl SockJ1939Filters {
    #[must_use]
    pub fn new() -> Self {
        // J1939 filter have at least one filter
        SockJ1939Filters { filter: Vec::new(), fastpkg: Vec::new() }
    }

    #[must_use]
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

    pub fn add_fast(&mut self, pgn: u32, dbc_len: u16, capacity: u16) -> &mut Self {
        self.fastpkg.push(RefCell::new(SockJ1939Fast::new(pgn, dbc_len, capacity)));
        self.add_pgn(pgn);
        self
    }
    /// Searches the internal PGN map and returns a mutable handle to the
    /// fast-path entry for the given Parameter Group Number (PGN).
    ///
    /// # Parameters
    /// - `pgn`: The J1939 Parameter Group Number to look up.
    ///
    /// # Returns
    /// A `RefMut<'_, SockJ1939Fast>` giving mutable access to the cached
    /// fast-path data associated with `pgn`.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the `pgn` is unknown or not present in the pool;
    /// - the internal map/index is not initialized or corrupted;
    /// - there is an outstanding borrow that prevents acquiring a mutable
    ///   reference (e.g., `RefCell` borrow conflict).
    pub fn search_pgn(&self, pgn: u32) -> Result<RefMut<'_, SockJ1939Fast>, CanError> {
        let search = self.fastpkg.binary_search_by(|pkg| pkg.borrow().pgn.cmp(&pgn));
        match search {
            Ok(idx) => match self.fastpkg[idx].try_borrow_mut() {
                Err(_code) => Err(CanError::new("message-get_mut", "internal fastpkg pool error")),
                Ok(mut_ref) => Ok(mut_ref),
            },
            Err(_) => Err(CanError::new("fail-fastpgn-search", format!("canid:{pgn} not found"))),
        }
    }

    pub fn add_name(&mut self, name: u64) -> &mut Self {
        let mut filter = unsafe { mem::zeroed::<cglue::j1939_filter>() };
        filter.name = name;
        filter.name_mask = !0;
        self.filter.push(filter);
        self
    }
    /// Applies the current J1939 filter configuration to the given socket.
    ///
    /// This configures the underlying CAN/J1939 socket with the filter(s)
    /// represented by this structure (e.g., PGN/address masks, promiscuous
    /// mode, timestamping flags), replacing any previous configuration.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the socket option calls (`setsockopt`, `bind`, etc.) fail (e.g., due to
    ///   insufficient privileges, an invalid descriptor, or OS-level errors);
    /// - the filter set is empty or internally inconsistent (e.g., invalid mask
    ///   widths or mutually exclusive flags);
    /// - required conversions (lengths, pointer casts) fail validation;
    /// - there is an internal borrow/state conflict that prevents applying the
    ///   configuration.
    pub fn apply(&mut self, sock: &SockCanHandle) -> Result<(), CanError> {
        // sort fast packet vector list
        self.fastpkg.sort_by(|a, b| a.borrow().pgn.cmp(&b.borrow().pgn));

        // build filter list
        let filter_len = self.filter.len();
        let j1939_filter = self.filter.as_slice();
        if filter_len > cglue::can_J1939_x_FILTER_MAX as usize {
            return Err(CanError::new("j1939-filter-number", "to many j1939 filters"));
        }

        // register filter list
        let sol_can_j1939: i32 =
            i32::try_from(cglue::can_J1939_x_SOL_CAN_J1939).unwrap_or(i32::MAX);
        let so_filter: i32 = i32::try_from(cglue::can_J1939_x_SO_FILTER).unwrap_or(i32::MAX);
        let opt_ptr = j1939_filter.as_ptr().cast::<std::ffi::c_void>();
        let opt_len: cglue::socklen_t =
            cglue::socklen_t::try_from(mem::size_of::<cglue::j1939_filter>() * filter_len)
                .unwrap_or(u32::MAX);

        let status =
            unsafe { cglue::setsockopt(sock.sockfd, sol_can_j1939, so_filter, opt_ptr, opt_len) };

        if status < 0 {
            return Err(CanError::new("fail-j1939-filter", cglue::get_perror()));
        }
        Ok(())
    }
}

#[inline]
#[cfg_attr(not(test), allow(dead_code))]
fn mask_eff_flag(id: u32) -> u32 {
    // Drop the CAN_EFF_FLAG (0x8000_0000) if present; keep payload bits.
    id & 0x7FFF_FFFF
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mask_eff_flag() {
        assert_eq!(mask_eff_flag(0x98A8_4444), 0x18A8_4444);
        assert_eq!(mask_eff_flag(0x18A8_4444), 0x18A8_4444);
    }
}
