/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
 * References:
 *    https://www.kernel.org/doc/html/latest/networking/can.html#broadcast-manager-protocol-sockets-sock-dgram
 *    https://www.kernel.org/doc/html/latest/networking/can.html#broadcast-manager-receive-filter-timers
 *
*/
use bitflags::bitflags;
use std::ffi::CStr;

use super::cglue;
use std::mem::{self, MaybeUninit};
use crate::prelude::*;

type SockCanId = cglue::canid_t;
bitflags! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct FilterMask: cglue::canid_t {
        /// SFF_MASK valid bits in standard frame id
        const SFF_MASK = cglue::can_MASK_x_SFF_MASK;
        /// EFF_MASK valid bits in extended frame id
        const EFF_MASK = cglue::can_MASK_x_EFF_MASK;
        /// EFF_FLAG indicate 29 bit extended format
        const EFF_FLAG= cglue::can_FLAGS_x_EFF_FLAG;
        /// RTR_FLAG remote transmission request flag
        const RTR_FLAG= cglue::can_FLAGS_x_RTR_FLAG;
        /// ERR_FLAG error flag
        const ERR_FLAG= cglue::can_FLAGS_x_ERR_FLAG;
        /// ERR_MASK valid bits in error frame
        const ERR_MASK=cglue::can_MASK_x_ERR_MASK;
    }
}

bitflags! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct CanErrorMask: cglue::canid_t {
        const TX_TIMEOUT = cglue::can_ERROR_x_TX_TIMEOUT;
        const BUS_OFF = cglue::can_ERROR_x_BUSOFF;
        const BUS_ERROR= cglue::can_ERROR_x_BUSERROR;
        const BUS_RESTARTED= cglue::can_ERROR_x_RESTARTED;
    }
}

bitflags! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct CanBcmFlag: u32 {
        const SET_TIMER          = cglue::can_BCM_FLAG_x_SETTIMER;
        const START_TIMER        = cglue::can_BCM_FLAG_x_STARTTIMER;
        const TX_COUNTEVT       = cglue::can_BCM_FLAG_x_TX_COUNTEVT;
        const TX_ANNOUNCE       = cglue::can_BCM_FLAG_x_TX_ANNOUNCE;
        const TX_CP_CAN_ID      = cglue::can_BCM_FLAG_x_TX_CP_CAN_ID;
        const RX_FILTER_ID      = cglue::can_BCM_FLAG_x_RX_FILTER_ID;
        const RX_CHECK_DLC      = cglue::can_BCM_FLAG_x_RX_CHECK_DLC;
        const RX_NO_AUTOTIMER   = cglue::can_BCM_FLAG_x_RX_NO_AUTOTIMER;
        const RX_ANNOUNCE_RESUME= cglue::can_BCM_FLAG_x_RX_ANNOUNCE_RESUME;
        const TX_RESET_MULTI_IDX= cglue::can_BCM_FLAG_x_TX_RESET_MULTI_IDX;
        const RX_RTR_FRAME      = cglue::can_BCM_FLAG_x_RX_RTR_FRAME;
        const FD_FRAME          = cglue::can_BCM_FLAG_x_FD_FRAME;
    }
}
impl CanBcmFlag {
    pub fn check(flags: CanBcmFlag, value: u32) -> bool {
        flags.bits() & value != 0
    }
}

#[derive(Debug)]
pub enum CanTimeStamp {
    NONE,
    CLASSIC,
    NANOSEC,
    HARDWARE,
    SOFTWARE,
}

impl CanBcmOpCode {
    pub fn from(opcode: u32) -> Result<CanBcmOpCode, CanError> {
        let ope = match opcode {
            cglue::can_BCM_OPE_x_TX_SETUP => CanBcmOpCode::TxSetup,
            cglue::can_BCM_OPE_x_TX_DELETE => CanBcmOpCode::TxDelete,
            cglue::can_BCM_OPE_x_TX_READ => CanBcmOpCode::TxRead,
            cglue::can_BCM_OPE_x_TX_SEND => CanBcmOpCode::TxSend,
            cglue::can_BCM_OPE_x_TX_STATUS => CanBcmOpCode::TxStatus,
            cglue::can_BCM_OPE_x_TX_EXPIRED => CanBcmOpCode::TxExpired,
            cglue::can_BCM_OPE_x_RX_SETUP => CanBcmOpCode::RxSetup,
            cglue::can_BCM_OPE_x_RX_DELETE => CanBcmOpCode::RxDelete,
            cglue::can_BCM_OPE_x_RX_READ => CanBcmOpCode::RxRead,
            cglue::can_BCM_OPE_x_RX_STATUS => CanBcmOpCode::RxStatus,
            cglue::can_BCM_OPE_x_RX_TIMEOUT => CanBcmOpCode::RxTimeout,
            cglue::can_BCM_OPE_x_RX_CHANGED => CanBcmOpCode::RxChanged,
            _ => {
                return Err(CanError::new(
                    "invalid-bcm-opcode",
                    format!("value={}", opcode),
                ))
            }
        };
        Ok(ope)
    }

    pub fn as_u32(opcode: &CanBcmOpCode) -> u32 {
        match opcode {
            CanBcmOpCode::TxSetup => cglue::can_BCM_OPE_x_TX_SETUP,
            CanBcmOpCode::TxDelete => cglue::can_BCM_OPE_x_TX_DELETE,
            CanBcmOpCode::TxRead => cglue::can_BCM_OPE_x_TX_READ,
            CanBcmOpCode::TxSend => cglue::can_BCM_OPE_x_TX_SEND,
            CanBcmOpCode::TxStatus => cglue::can_BCM_OPE_x_TX_STATUS,
            CanBcmOpCode::TxExpired => cglue::can_BCM_OPE_x_TX_EXPIRED,
            CanBcmOpCode::RxSetup => cglue::can_BCM_OPE_x_RX_SETUP,
            CanBcmOpCode::RxDelete => cglue::can_BCM_OPE_x_RX_DELETE,
            CanBcmOpCode::RxRead => cglue::can_BCM_OPE_x_RX_READ,
            CanBcmOpCode::RxStatus => cglue::can_BCM_OPE_x_RX_STATUS,
            CanBcmOpCode::RxTimeout => cglue::can_BCM_OPE_x_RX_TIMEOUT,
            CanBcmOpCode::RxChanged => cglue::can_BCM_OPE_x_RX_CHANGED,
            CanBcmOpCode::Unknown => 0xFFFF,
        }
    }
}

/// Classical CAN frame structure (aka CAN 2.0B)
/// canid:   CAN ID of the frame and CAN_///_FLAG flags, see canid_t definition
/// @len:      CAN frame payload length in byte (0 .. 8)
/// @can_dlc:  deprecated name for CAN frame payload length in byte (0 .. 8)
/// @__pad:    padding
/// @__res0:   reserved / padding
/// @len8_dlc: optional DLC value (9 .. 15) at 8 byte payload length
///            len8_dlc contains values from 9 .. 15 when the payload length is
///            8 bytes but the DLC value (see ISO 11898-1) is greater then 8.
///            CAN_CTRLMODE_CC_LEN8_DLC flag has to be enabled in CAN driver.
/// @data:     CAN frame payload (up to 8 byte)
pub struct CanFrameRaw(cglue::can_frame);
impl CanFrameRaw {
    pub fn new(canid: SockCanId, len: u8, pad: u8, res: u8, data: [u8; 8usize]) -> Self {
        CanFrameRaw {
            0: cglue::can_frame {
                can_id: canid,
                __pad: pad,
                __res0: res,
                len8_dlc: 0,
                data: data,
                __bindgen_anon_1: cglue::can_frame__bindgen_ty_1 { len: len },
            },
        }
    }
    pub fn empty(canid: u32) -> Self {
        let mut frame: CanFrameRaw = unsafe { mem::zeroed::<Self>() };
        frame.0.can_id = canid;
        frame
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        &self.0 as *const _ as *mut std::ffi::c_void
    }

    pub fn get_id(&self) -> SockCanId {
        self.0.can_id as SockCanId
    }

    pub fn get_len(&self) -> u8 {
        unsafe { self.0.__bindgen_anon_1.len }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.0.data
    }
}
pub struct CanFdFrameRaw(cglue::canfd_frame);
impl CanFdFrameRaw {
    pub fn new(
        canid: SockCanId,
        len: u8,
        flags: u8,
        res0: u8,
        res1: u8,
        data: [u8; 64usize],
    ) -> Self {
        CanFdFrameRaw {
            0: cglue::canfd_frame {
                can_id: canid,
                len: len,
                __res0: res0,
                __res1: res1,
                flags: flags,
                data: data,
            },
        }
    }

    pub fn empty(canid: u32) -> Self {
        let mut frame: CanFdFrameRaw = unsafe { mem::zeroed::<Self>() };
        frame.0.can_id = canid;
        frame
    }

    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        &self.0 as *const _ as *mut std::ffi::c_void
    }

    pub fn get_id(&self) -> SockCanId {
        self.0.can_id as SockCanId
    }

    pub fn get_len(&self) -> u8 {
        self.0.len
    }
    pub fn get_flag(&self) -> u8 {
        self.0.flags
    }

    pub fn get_data(&self) -> &[u8] {
        &self.0.data
    }
}

pub enum CanAnyFrame {
    RawFd(CanFdFrameRaw),
    RawStd(CanFrameRaw),
    Err(CanError),
    None(u32),
}

impl CanAnyFrame {
    pub fn get_id(&self) -> Result<u32, CanError> {
        match self {
            CanAnyFrame::RawFd(frame) => Ok(frame.get_id()),
            CanAnyFrame::RawStd(frame) => Ok(frame.get_id()),
            CanAnyFrame::Err(error) => Err(error.clone()),
            CanAnyFrame::None(canid) => Ok(*canid),
        }
    }

    pub fn get_len(&self) -> Result<u8, CanError> {
        match self {
            CanAnyFrame::RawFd(frame) => Ok(frame.get_len()),
            CanAnyFrame::RawStd(frame) => Ok(frame.get_len()),
            CanAnyFrame::Err(error) => Err(error.clone()),
            CanAnyFrame::None(_canid) => Ok(0),
        }
    }

    pub fn get_data(&self) -> Result<&[u8], CanError> {
        match self {
            CanAnyFrame::RawFd(frame) => Ok(frame.get_data()),
            CanAnyFrame::RawStd(frame) => Ok(frame.get_data()),
            CanAnyFrame::Err(error) => Err(error.clone()),
            CanAnyFrame::None(_canid) => Ok(&[0]),
        }
    }
}

impl From<CanError> for CanAnyFrame {
    fn from(frame: CanError) -> Self {
        Self::Err(frame)
    }
}

impl From<*const CanFrameRaw> for CanAnyFrame {
    fn from(src: *const CanFrameRaw) -> Self {
        #[allow(invalid_value)]
        let dst: CanFrameRaw = unsafe { MaybeUninit::uninit().assume_init() };
        unsafe { std::ptr::copy_nonoverlapping(src, &dst.0 as *const _ as *mut CanFrameRaw, 1) };
        CanAnyFrame::RawStd(dst)
    }
}

impl From<*const CanFdFrameRaw> for CanAnyFrame {
    fn from(src: *const CanFdFrameRaw) -> Self {
        #[allow(invalid_value)]
        let dst: CanFdFrameRaw = unsafe { MaybeUninit::uninit().assume_init() };
        unsafe { std::ptr::copy(src, &dst as *const _ as *mut CanFdFrameRaw, 1) };
        CanAnyFrame::RawFd(dst)
    }
}

struct CanBcmOneMsg(cglue::can_bcm_one_msg);
struct CanFdBcmOneMsg(cglue::canfd_bcm_one_msg);
struct CanBcmHeader(cglue::bcm_msg_head);

pub struct SockCanMsg {
    iface: i32,
    stamp: u64,
    frame: CanAnyFrame,
}

impl SockCanMsg {
    pub fn get_iface(&self) -> i32 {
        self.iface
    }

    pub fn get_stamp(&self) -> u64 {
        self.stamp
    }

    pub fn get_raw(&self) -> &CanAnyFrame {
        &self.frame
    }

    pub fn get_ifname(&self, sock: &SockCanHandle) -> Result<String, CanError> {
        let mut ifreq: cglue::ifreq = unsafe { mem::MaybeUninit::uninit().assume_init() };
        ifreq.ifr_ifru.ifru_ivalue /* ifr_index */= self.iface;
        let rc = unsafe { cglue::ioctl(sock.sockfd, cglue::can_SOCK_x_SIOCGIFNAME as u64, &ifreq) };

        if rc < 0 {
            Err(CanError::new("can-ifname-fail", cglue::get_perror()))
        } else {
            let cstring = unsafe { CStr::from_ptr(&ifreq.ifr_ifrn.ifrn_name as *const i8) };
            match cstring.to_str() {
                Err(error) => Err(CanError::new("can-ifname-invalid", error.to_string())),
                Ok(slice) => Ok(slice.to_owned()),
            }
        }
    }

    pub fn get_len(&self) -> Result<u8, CanError> {
        self.frame.get_len()
    }

    pub fn get_id(&self) -> Result<u32, CanError> {
        self.frame.get_id()
    }

    pub fn get_data(&self) -> Result<&[u8], CanError> {
        self.frame.get_data()
    }
}

pub struct SockBcmMsg {
    opcode: CanBcmOpCode,
    iface: i32,
    stamp: u64,
    frame: CanAnyFrame,
}

impl SockBcmMsg {
    pub fn get_iface(&self) -> i32 {
        self.iface
    }

    pub fn get_opcode(&self) -> CanBcmOpCode {
        self.opcode
    }

    pub fn get_stamp(&self) -> u64 {
        self.stamp
    }

    pub fn get_raw(&self) -> &CanAnyFrame {
        &self.frame
    }

    pub fn get_ifname(&self, sock: &SockCanHandle) -> Result<String, CanError> {
        let mut ifreq: cglue::ifreq = unsafe { mem::MaybeUninit::uninit().assume_init() };
        ifreq.ifr_ifru.ifru_ivalue /* ifr_index */= self.iface;
        let rc = unsafe { cglue::ioctl(sock.sockfd, cglue::can_SOCK_x_SIOCGIFNAME as u64, &ifreq) };

        if rc < 0 {
            Err(CanError::new("can-ifname-fail", cglue::get_perror()))
        } else {
            let cstring = unsafe { CStr::from_ptr(&ifreq.ifr_ifrn.ifrn_name as *const i8) };
            match cstring.to_str() {
                Err(error) => Err(CanError::new("can-ifname-invalid", error.to_string())),
                Ok(slice) => Ok(slice.to_owned()),
            }
        }
    }

    pub fn get_len(&self) -> Result<u8, CanError> {
        self.frame.get_len()
    }

    pub fn get_id(&self) -> Result<u32, CanError> {
        self.frame.get_id()
    }

    pub fn get_data(&self) -> Result<&[u8], CanError> {
        self.frame.get_data()
    }
}

pub struct SockCanFilter {
    count: usize,
    masks: Vec<cglue::can_filter>,
}

enum SockCanMod {
    RAW,
    BCM,
    _ISOTP,
    _J1939,
    _NMEA2000,
}

struct CanRecvInfo {
    stamp: u64,
    count: isize,
    iface: i32,
}

pub struct SockCanHandle {
    sockfd: ::std::os::raw::c_int,
    mode: SockCanMod,
}

pub trait CanIFaceFrom<T> {
    fn map_can_iface(sock: i32, iface: T) -> i32;
}

impl CanIFaceFrom<&str> for SockCanHandle {
    fn map_can_iface(sock: i32, iface: &str) -> i32 {
        let mut ifreq: cglue::ifreq = unsafe { mem::zeroed() };

        let iname = iface.as_bytes();

        for idx in 0..cglue::can_SOCK_x_IFACE_LEN as usize {
            if idx == iface.len() {
                break;
            };
            unsafe { ifreq.ifr_ifrn.ifrn_name[idx] = iname[idx] as ::std::os::raw::c_char };
        }

        // get Can iface index
        let rc = unsafe { cglue::ioctl(sock, cglue::can_SOCK_x_SIOCGIFINDEX as u64, &ifreq) };

        if rc < 0 {
            rc
        } else {
            unsafe { ifreq.ifr_ifru.ifru_ivalue } //ifr.ifr_if index
        }
    }
}

impl CanIFaceFrom<u32> for SockCanHandle {
    fn map_can_iface(_sock: i32, iface: u32) -> i32 {
        iface as i32
    }
}

impl SockCanHandle {

    pub fn open_raw<T>(candev: T, timestamp: CanTimeStamp) -> Result<Self, CanError>
    where
        SockCanHandle: CanIFaceFrom<T>,
    {
        let sockfd = unsafe {
            cglue::socket(
                cglue::can_SOCK_x_PF_CAN as i32,
                cglue::can_SOCK_x_SOCK_RAW as i32,
                cglue::can_PF_x_RAW as i32,
            )
        };
        if sockfd < 0 {
            return Err(CanError::new("fail-socketcan-open", cglue::get_perror()));
        }

        let index = SockCanHandle::map_can_iface(sockfd, candev);
        if index < 0 {
            return Err(CanError::new("fail-socketcan-iface", cglue::get_perror()));
        }

        #[allow(invalid_value)]
        let mut canaddr: cglue::sockaddr_can = unsafe { MaybeUninit::uninit().assume_init() };
        canaddr.can_family = cglue::can_SOCK_x_PF_CAN as u16;
        canaddr.can_ifindex = index;

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
            return Err(CanError::new("fail-socketcan-bind", cglue::get_perror()));
        }

        let mut sockcan = SockCanHandle {
            sockfd: sockfd,
            mode: SockCanMod::RAW,
        };

        match sockcan.set_timestamp(timestamp) {
            Err(error) => return Err(error),
            Ok(_value) => {}
        }

        Ok(sockcan)
    }

    pub fn open_bcm<T>(candev: T, timestamp: CanTimeStamp) -> Result<Self, CanError>
    where
        SockCanHandle: CanIFaceFrom<T>,
    {
        let sockfd = unsafe {
            cglue::socket(
                cglue::can_SOCK_x_AF_CAN as i32,
                cglue::can_SOCK_x_SOCK_DGRAM as i32,
                cglue::can_PF_x_BCM as i32,
            )
        };
        if sockfd < 0 {
            return Err(CanError::new("fail-socketcan-open", cglue::get_perror()));
        }

        let index = SockCanHandle::map_can_iface(sockfd, candev);
        if index < 0 {
            return Err(CanError::new("fail-socketcan-iface", cglue::get_perror()));
        }

        #[allow(invalid_value)]
        let mut canaddr: cglue::sockaddr_can = unsafe { MaybeUninit::uninit().assume_init() };
        canaddr.can_family = cglue::can_SOCK_x_AF_CAN as u16;
        canaddr.can_ifindex = index;

        let sockaddr = cglue::__CONST_SOCKADDR_ARG {
            __sockaddr__: &canaddr as *const _ as *const cglue::sockaddr,
        };
        let status = unsafe {
            cglue::connect(
                sockfd,
                sockaddr,
                mem::size_of::<cglue::sockaddr_can>() as cglue::socklen_t,
            )
        };
        if status < 0 {
            return Err(CanError::new("fail-socketcan-connect", cglue::get_perror()));
        }

        let mut sockcan = SockCanHandle {
            sockfd: sockfd,
            mode: SockCanMod::BCM,
        };

        match sockcan.set_timestamp(timestamp) {
            Err(error) => return Err(error),
            Ok(_value) => {}
        }

        Ok(sockcan)
    }

    pub fn close (&self) {
       unsafe {cglue::close(self.sockfd)};
    }

    pub fn as_rawfd (&self) -> i32 {
       self.sockfd
    }

    pub fn set_blocking(&mut self, blocking: bool) -> Result<&mut Self, CanError> {
        // retrieve current flags
        let current_flag = unsafe { cglue::fcntl(self.sockfd, cglue::can_SOCK_x_F_GETFL as i32) };
        if current_flag < 0 {
            return Err(CanError::new("can-nonblock-fail", cglue::get_perror()));
        }

        let new_flag = if blocking {
            current_flag & !cglue::can_SOCK_x_NONBLOCK as i32
        } else {
            current_flag | cglue::can_SOCK_x_NONBLOCK as i32
        };

        let status =
            unsafe { cglue::fcntl(self.sockfd, cglue::can_SOCK_x_F_SETFL as i32, new_flag) };
        if status < 0 {
            return Err(CanError::new("can-nonblock-fail", cglue::get_perror()));
        }
        Ok(self)
    }

    pub fn set_timeout(&mut self, read_ms: i64, write_ms: i64) -> Result<&mut Self, CanError> {
        if read_ms > 0 {
            let timout = cglue::timeval {
                tv_sec: read_ms / 1000,
                tv_usec: read_ms * 1000 % 1000000,
            };
            unsafe {
                let status = cglue::setsockopt(
                    self.sockfd,
                    cglue::can_SOCK_x_SOL_SOCKET as ::std::os::raw::c_int,
                    cglue::can_SOCK_x_SO_RCVTIMEO as ::std::os::raw::c_int,
                    &timout as *const _ as *const ::std::os::raw::c_void,
                    mem::size_of::<cglue::timeval>() as cglue::socklen_t,
                );

                if status < 0 {
                    return Err(CanError::new("can-read-fail", cglue::get_perror()));
                }
            }
        }

        if write_ms > 0 {
            let timout = cglue::timeval {
                tv_sec: write_ms / 1000,
                tv_usec: write_ms * 1000 % 1000000,
            };
            unsafe {
                let status = cglue::setsockopt(
                    self.sockfd,
                    cglue::can_SOCK_x_SOL_SOCKET as ::std::os::raw::c_int,
                    cglue::can_SOCK_x_SO_SNDTIMEO as ::std::os::raw::c_int,
                    &timout as *const _ as *const ::std::os::raw::c_void,
                    mem::size_of::<cglue::timeval>() as cglue::socklen_t,
                );

                if status < 0 {
                    return Err(CanError::new("can-read-fail", cglue::get_perror()));
                }
            }
        }

        Ok(self)
    }

    pub fn set_recv_own(&mut self, loopback: bool) -> Result<&mut Self, CanError> {
        let flag = if loopback { 1 } else { 0 };
        let status = unsafe {
            cglue::setsockopt(
                self.sockfd,
                cglue::can_RAW_x_SOL_CAN_RAW as i32,
                cglue::can_RAW_x_RECV_OWN_MSGS as i32,
                &flag as *const _ as *const std::ffi::c_void,
                mem::size_of::<i32>() as u32,
            )
        };
        if status < 0 {
            return Err(CanError::new("can-recv-own-fail", cglue::get_perror()));
        }
        Ok(self)
    }

    fn set_timestamp(&mut self, timestamp: CanTimeStamp) -> Result<&mut Self, CanError> {
        let status = match timestamp {
            CanTimeStamp::SOFTWARE => {
                let flag = cglue::can_RAW_x_SOF_TIMESTAMPING_RX_SOFTWARE;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        cglue::can_SOCK_x_SOL_SOCKET as i32,
                        cglue::can_RAW_x_SO_TIMESTAMPING as i32,
                        &flag as *const _ as *const std::ffi::c_void,
                        mem::size_of::<i32>() as u32,
                    )
                }
            }
            CanTimeStamp::HARDWARE => {
                let flag = cglue::can_RAW_x_SOF_TIMESTAMPING_RX_HARDWARE;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        cglue::can_SOCK_x_SOL_SOCKET as i32,
                        cglue::can_RAW_x_SO_TIMESTAMPING as i32,
                        &flag as *const _ as *const std::ffi::c_void,
                        mem::size_of::<i32>() as u32,
                    )
                }
            }
            CanTimeStamp::NANOSEC => {
                let flag = 1 as u32;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        cglue::can_SOCK_x_SOL_SOCKET as i32,
                        cglue::can_RAW_x_SO_TIMESTAMPNS as i32,
                        &flag as *const _ as *const std::ffi::c_void,
                        mem::size_of::<i32>() as u32,
                    )
                }
            }
            CanTimeStamp::CLASSIC => {
                let flag = 1 as u32;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        cglue::can_SOCK_x_SOL_SOCKET as i32,
                        cglue::can_RAW_x_SO_TIMESTAMP as i32,
                        &flag as *const _ as *const std::ffi::c_void,
                        mem::size_of::<i32>() as u32,
                    )
                }
            }
            CanTimeStamp::NONE => 0,
        };

        if status < 0 {
            return Err(CanError::new("can-setsock-stamp-fail", cglue::get_perror()));
        }
        Ok(self)
    }

    pub fn set_monitoring(&mut self, mask: CanErrorMask) -> Result<&mut Self, CanError> {
        let flag = mask.bits();
        let status = unsafe {
            cglue::setsockopt(
                self.sockfd,
                cglue::can_RAW_x_SOL_CAN_RAW as i32,
                mask.bits() as i32,
                &flag as *const _ as *const std::ffi::c_void,
                mem::size_of::<u32>() as u32,
            )
        };
        if status < 0 {
            return Err(CanError::new("can-setsock-err-fail", cglue::get_perror()));
        }
        Ok(self)
    }

    pub fn get_raw_frame(&self) -> CanAnyFrame {
        #[allow(invalid_value)]
        let buffer: [u8; cglue::can_MTU_x_FD_MTU as usize] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let count = unsafe {
            cglue::read(
                self.sockfd,
                &buffer as *const _ as *mut std::ffi::c_void,
                cglue::can_MTU_x_FD_MTU as usize,
            )
        };

        if count == mem::size_of::<CanFrameRaw>() as isize {
            CanAnyFrame::from(&buffer as *const _ as *const CanFrameRaw)
        } else if count == mem::size_of::<CanFdFrameRaw>() as isize {
            CanAnyFrame::from(&buffer as *const _ as *const CanFdFrameRaw)
        } else {
            CanAnyFrame::Err(CanError::new("can-invalid-frame", cglue::get_perror()))
        }
    }

    fn recv_can_msg(&self, buffer: &mut [u8], size: u32) -> CanRecvInfo {
        let iovec = cglue::iovec {
            iov_base: buffer as *const _ as *mut std::ffi::c_void,
            iov_len: size as usize,
        };

        #[allow(invalid_value)]
        let control: cglue::can_stamp_msg = unsafe { MaybeUninit::uninit().assume_init() };

        #[allow(invalid_value)]
        let canaddr: cglue::sockaddr_can = unsafe { MaybeUninit::uninit().assume_init() };

        #[allow(invalid_value)]
        let mut msg_hdr: cglue::msghdr = unsafe { mem::zeroed() };
        msg_hdr.msg_flags = 0;
        msg_hdr.msg_iov = &iovec as *const _ as *mut cglue::iovec;
        msg_hdr.msg_iovlen = 1;
        msg_hdr.msg_namelen = mem::size_of::<cglue::sockaddr_can>() as cglue::socklen_t;
        msg_hdr.msg_name = &canaddr as *const _ as *mut std::ffi::c_void;
        msg_hdr.msg_control = &control as *const _ as *mut std::ffi::c_void;
        msg_hdr.msg_controllen = mem::size_of::<cglue::can_stamp_msg>();

        let count =
            unsafe { cglue::recvmsg(self.sockfd, &msg_hdr as *const _ as *mut cglue::msghdr, 0) };

        let iface = if msg_hdr.msg_namelen >= 8 {
            let canaddr = unsafe { &*(msg_hdr.msg_name as *const _ as *mut cglue::sockaddr_can) };
            canaddr.can_ifindex
        } else {
            0
        };

        // ref: https://github.com/torvalds/linux/blob/master/tools/testing/selftests/net/timestamping.c
        let cmsg = unsafe { &*cglue::CMSG_FIRSTHDR(&msg_hdr) };
        let mut stamp = 0;
        while cmsg as *const _ != 0 as *const _ {
            if cmsg.cmsg_level == cglue::can_SOCK_x_SOL_SOCKET as i32 {
                match cmsg.cmsg_type as u32 {
                    cglue::can_RAW_x_SO_TIMESTAMPING => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timespec)
                        };
                        stamp = (time.tv_sec * 1000000 + time.tv_nsec) as u64;
                        break;
                    }
                    cglue::can_RAW_x_SO_TIMESTAMP_NEW => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timespec)
                        };
                        stamp = (time.tv_sec * 1000000 + time.tv_nsec) as u64;
                        break;
                    }
                    cglue::can_RAW_x_SO_TIMESTAMPNS => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timespec)
                        };
                        stamp = (time.tv_sec * 1000000 + time.tv_nsec) as u64;
                        break;
                    }
                    cglue::can_RAW_x_SO_TIMESTAMP => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timeval)
                        };
                        stamp = (time.tv_sec * 1000000 + time.tv_usec) as u64;
                        break;
                    }
                    _ => {}
                }
            }
        }
        CanRecvInfo {
            stamp: stamp,
            count: count,
            iface: iface,
        }
    }

    pub fn get_can_frame(&self) -> SockCanMsg {
        #[allow(invalid_value)]
        let mut buffer: [u8; cglue::can_MTU_x_FD_MTU as usize] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let info = self.recv_can_msg(&mut buffer, cglue::can_MTU_x_FD_MTU);

        let can_any_frame = if info.count == mem::size_of::<CanFrameRaw>() as isize {
            CanAnyFrame::from(&buffer as *const _ as *const CanFrameRaw)
        } else if info.count == mem::size_of::<CanFdFrameRaw>() as isize {
            CanAnyFrame::from(&buffer as *const _ as *const CanFdFrameRaw)
        } else {
            CanAnyFrame::Err(CanError::new("can-invalid-frame", cglue::get_perror()))
        };

        SockCanMsg {
            frame: can_any_frame,
            iface: info.iface,
            stamp: info.stamp,
        }
    }

    pub fn get_bcm_frame(&self) -> SockBcmMsg {

        #[allow(invalid_value)]
        let mut buffer: [u8; mem::size_of::<CanFdBcmOneMsg>()] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let info = self.recv_can_msg(&mut buffer, mem::size_of::<CanFdBcmOneMsg>() as u32);
        let one_msg = unsafe { &*(&buffer as *const _ as *const CanBcmOneMsg) };

        let can_any_frame = if info.count == mem::size_of::<CanBcmOneMsg>() as isize {
            CanAnyFrame::from(&one_msg.0.frame as *const _ as *const CanFrameRaw)
        } else if info.count == mem::size_of::<CanFdBcmOneMsg>() as isize {
            let one_msg = unsafe { &*(&buffer as *const _ as *const CanFdBcmOneMsg) };
            CanAnyFrame::from(&one_msg.0.frame as *const _ as *const CanFdFrameRaw)
        } else if info.count == mem::size_of::<CanBcmHeader>() as isize {
            let header = unsafe { &*(&buffer as *const _ as *const CanBcmHeader) };
            CanAnyFrame::None(header.0.can_id)
        } else {
            CanAnyFrame::Err(CanError::new("bcm-invalid-frame", cglue::get_perror()))
        };

        let opcode = match CanBcmOpCode::from(one_msg.0.head.opcode) {
            Ok(value) => value,
            Err(_error) => CanBcmOpCode::Unknown,
        };

        SockBcmMsg {
            opcode: opcode,
            frame: can_any_frame,
            iface: info.iface,
            stamp: info.stamp,
        }
    }
}

impl SockCanFilter {
    pub fn new(size: usize) -> Self {
        if size > 0 {
            SockCanFilter {
                count: 0,
                masks: Vec::with_capacity(size),
            }
        } else {
            SockCanFilter {
                count: 0,
                masks: Vec::new(),
            }
        }
    }

    /// Each filter contains an internal id and mask. Packets are considered to be matched
    /// by a filter if `received_id & mask == filter_id & mask` holds true.
    pub fn add_whitelist(&mut self, can_id: u32, can_mask: FilterMask) -> &mut Self {
        self.count += 1;
        self.masks.push(cglue::can_filter {
            can_id: can_id,
            can_mask: can_mask.bits(),
        });
        self
    }

    pub fn add_blacklist(&mut self, can_id: u32, can_mask: FilterMask) -> &mut Self {
        self.count += 1;
        self.masks.push(cglue::can_filter {
            can_id: can_id | cglue::can_FILTER_x_INV_FILTER,
            can_mask: can_mask.bits(),
        });
        self
    }

    pub fn apply(&mut self, sock: &SockCanHandle) -> Result<(), CanError> {
        match sock.mode {
            SockCanMod::RAW => {}
            _ => {
                return Err(CanError::new(
                    "invalid-socketcan-mod",
                    "not a RAW socket can",
                ))
            }
        }

        let filter_ptr = self.masks.as_ptr();
        let status = unsafe {
            cglue::setsockopt(
                sock.sockfd,
                cglue::can_RAW_x_SOL_CAN_RAW as ::std::os::raw::c_int,
                cglue::can_RAW_x_FILTER as ::std::os::raw::c_int,
                filter_ptr as *const ::std::os::raw::c_void,
                (mem::size_of::<CanFrameRaw>() * self.count) as cglue::socklen_t,
            )
        };
        if status < 0 {
            return Err(CanError::new("fail-socketcan-bind", cglue::get_perror()));
        } else {
            Ok(())
        }
    }
}

// struct bcm_msg
// {
// 	union {
// 		struct canfd_frame fd_frames[MAX_BCM_CAN_FRAMES];
// 		struct can_frame frames[MAX_BCM_CAN_FRAMES];
// 	};
// 	struct bcm_msg_head msg_head;
// };

pub struct CanBcmMsg(cglue::can_bcm_msg);
impl CanBcmMsg {
    pub fn empty() -> CanBcmMsg {
        unsafe { mem::zeroed::<Self>() }
    }

    pub fn as_raw(&self) -> *mut std::ffi::c_void {
        &self.0 as *const _ as *mut std::ffi::c_void
    }

    pub fn get_count(&self) -> u32 {
        self.0.head.count
    }

    pub fn get_id(&self) -> u32 {
        self.0.head.can_id
    }

    pub fn check_flags(&self, flags: CanBcmFlag) -> bool {
        flags.bits() & self.0.head.flags != 0
    }
}

pub struct CanFdBcmMsg(cglue::canfd_bcm_msg);
impl CanFdBcmMsg {
    pub fn empty() -> CanFdBcmMsg {
        unsafe { mem::zeroed::<Self>() }
    }

    pub fn as_raw(&self) -> *mut std::ffi::c_void {
        &self.0 as *const _ as *mut std::ffi::c_void
    }

    pub fn get_count(&self) -> u32 {
        self.0.head.count
    }

    pub fn get_id(&self) -> u32 {
        self.0.head.can_id
    }

    pub fn check_flags(&self, flags: CanBcmFlag) -> bool {
        flags.bits() & self.0.head.flags != 0
    }
}

pub struct SockBcmCmd {
    opcode: CanBcmOpCode,
    flags: CanBcmFlag,
    rx_watchdog: u64,
    rx_maxrate: u64,
    canid: SockCanId,
    frames: Vec<CanFrameRaw>,
    fdframes: Vec<CanFdFrameRaw>,
    muxid: Vec<SockCanId>,
}

pub trait CanBcmAddFilter<T> {
    fn add_filter(&mut self, filter: T) -> &mut Self;
}

impl CanBcmAddFilter<SockCanId> for SockBcmCmd {
    fn add_filter(&mut self, filter: SockCanId) -> &mut Self {
        self.muxid.push(filter);
        self
    }
}

impl CanBcmAddFilter<CanFrameRaw> for SockBcmCmd {
    fn add_filter(&mut self, filter: CanFrameRaw) -> &mut Self {
        self.frames.push(filter);
        self
    }
}

impl CanBcmAddFilter<CanFdFrameRaw> for SockBcmCmd {
    fn add_filter(&mut self, filter: CanFdFrameRaw) -> &mut Self {
        self.fdframes.push(filter);
        self
    }
}

impl SockBcmCmd {
    pub fn new(opcode: CanBcmOpCode, flags: CanBcmFlag, canid: SockCanId) -> Self {
        // BCM filter have at least one filter
        SockBcmCmd {
            opcode: opcode,
            flags: flags,
            canid: canid,
            frames: Vec::new(),
            fdframes: Vec::new(),
            muxid: Vec::new(),
            rx_maxrate: 0,
            rx_watchdog: 0,
        }
    }

    pub fn set_timers(&mut self, rx_maxrate: u64, rx_watchdog: u64) -> &mut Self {
        self.rx_watchdog = rx_watchdog;
        self.rx_maxrate = rx_maxrate;
        self
    }

    pub fn add_multiplex<T>(&mut self, filter: T) -> &mut Self
    where
        SockBcmCmd: CanBcmAddFilter<T>,
    {
        self.add_filter(filter);
        self
    }

    fn msg_head(&self, head: &mut cglue::bcm_msg_head) {
        head.opcode = CanBcmOpCode::as_u32(&self.opcode);
        head.can_id = self.canid;
        head.flags = self.flags.bits();
        head.nframes = 0;

        if self.rx_watchdog > 0 {
            head.ival1 = cglue::bcm_timeval {
                tv_sec: (self.rx_watchdog / 1000) as i64,
                tv_usec: (self.rx_watchdog * 1000 % 1000000) as i64,
            };
        }

        if self.rx_maxrate > 0 {
            head.ival2 = cglue::bcm_timeval {
                tv_sec: (self.rx_maxrate / 1000) as i64,
                tv_usec: (self.rx_maxrate * 1000 % 1000000) as i64,
            };
        }
    }

    pub fn apply(&mut self, sock: &SockCanHandle) -> Result<(), CanError> {
        match sock.mode {
            SockCanMod::BCM => {}
            _ => {
                return Err(CanError::new(
                    "invalid-socketcan-mod",
                    "not a BCM socketcan",
                ))
            }
        }

        match self.opcode {
            CanBcmOpCode::RxSetup => {
                if CanBcmFlag::check(CanBcmFlag::RX_FILTER_ID, self.flags.bits()) {
                    if self.frames.len() > 0 || self.fdframes.len() > 0 {
                        return Err(CanError::new(
                            "invalid-multiplex-filter",
                            "RX_FILTER_ID and Multiplex filter are exclusinve",
                        ));
                    }
                } else if CanBcmFlag::check(CanBcmFlag::FD_FRAME, self.flags.bits()) {
                    if self.frames.len() > 0 {
                        return Err(CanError::new(
                            "invalid-socketcan-filter",
                            "BCM:FdFrame no stdFrame mask allow",
                        ));
                    };
                    if self.fdframes.len() == 0 {
                        return Err(CanError::new(
                            "invalid-socketcan-filter",
                            "BCM:FdFrame no filter defined",
                        ));
                    };
                } else {
                    if self.frames.len() == 0 {
                        return Err(CanError::new(
                            "invalid-socketcan-filter",
                            "BCM:StdFrame no filter defined",
                        ));
                    };
                }
            }
            _ => {
                return Err(CanError::new(
                    "invalid-bcm-operation",
                    "bcm operation not yet implemented",
                ));
            }
        }

        // cglue::bcm_msg_head {
        //     opcode: cglue::can_BCM_x_RX_SETUP,
        //     can_id: self.canid,
        //     nframes: self.count as u32,
        //     flags: 0,
        //     count: 0,
        //     ival1: cglue::bcm_timeval { tv_sec:0, tv_usec:0},
        //     ival2: cglue::bcm_timeval { tv_sec:0, tv_usec:0},
        //     frames: cglue::__IncompleteArrayField::<cglue::can_frame>::new()
        // };

        let (buffer_addr, buffer_len) =
            if !CanBcmFlag::check(CanBcmFlag::FD_FRAME, self.flags.bits()) {
                // standard can messages
                #[allow(invalid_value)]
                let mut bcm_msg: cglue::can_bcm_msg =
                    unsafe { MaybeUninit::uninit().assume_init() };
                // haed is common to std and fd frames
                self.msg_head(&mut bcm_msg.head);

                for idx in 0..self.frames.len() {
                    bcm_msg.frames[idx] = self.frames[idx].0;
                }

                for idx in 0..self.muxid.len() {
                    bcm_msg.frames[idx] = CanFrameRaw::empty(self.muxid[idx]).0;
                }

                bcm_msg.head.nframes = (self.muxid.len() + self.frames.len()) as u32;
                let buffer = &bcm_msg as *const _ as *const ::std::os::raw::c_void;
                let len = mem::size_of::<cglue::bcm_msg_head>()
                    + bcm_msg.head.nframes as usize * mem::size_of::<cglue::can_frame>();

                (buffer, len)
            } else {
                // fdcan can messages
                #[allow(invalid_value)]
                let mut bcm_msg: cglue::canfd_bcm_msg =
                    unsafe { MaybeUninit::uninit().assume_init() };
                self.msg_head(&mut bcm_msg.head);

                for idx in 0..self.fdframes.len() {
                    bcm_msg.fdframes[idx] = self.fdframes[idx].0;
                }

                for idx in 0..self.muxid.len() {
                    bcm_msg.fdframes[idx] = CanFdFrameRaw::empty(self.muxid[idx]).0;
                }

                bcm_msg.head.nframes = (self.muxid.len() + self.frames.len()) as u32;
                let buffer = &bcm_msg as *const _ as *const ::std::os::raw::c_void;
                let len = mem::size_of::<cglue::bcm_msg_head>()
                    + bcm_msg.head.nframes as usize * mem::size_of::<cglue::can_frame>();

                (buffer, len)
            };

        let count = unsafe { cglue::write(sock.sockfd, buffer_addr, buffer_len) };
        if count != buffer_len as isize {
            return Err(CanError::new("fail-socketbcm-write", cglue::get_perror()));
        } else {
            Ok(())
        }
    }
}
