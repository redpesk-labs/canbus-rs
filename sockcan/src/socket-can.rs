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
 *    https://docs.kernel.org/networking/j1939.html
 *
*/
use bitflags::bitflags;
use std::ffi::CStr;

use super::cglue;
use crate::prelude::*;
use std::mem::{self, MaybeUninit};

pub type SockCanId = cglue::canid_t;
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

#[derive(Debug)]
pub enum CanTimeStamp {
    NONE,
    CLASSIC,
    NANOSEC,
    HARDWARE,
    SOFTWARE,
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
pub struct CanFrameRaw(pub cglue::can_frame);
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
pub struct CanFdFrameRaw(pub cglue::canfd_frame);
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

pub enum SockCanMod {
    RAW,
    BCM,
    J1939,
    _ISOTP,
}

#[derive(Clone, Copy)]
pub struct CanJ1939Header {
    pub name: u64,
    pub addr: u8,
}
#[derive(Clone, Copy)]
pub struct CanJ1939Info {
    pub src: CanJ1939Header,
    pub dst: CanJ1939Header,
    pub pgn: u32,
}

#[derive(Clone, Copy)]
pub struct CanIsoTpInfo {
    pub src: CanJ1939Header,
    pub dst: CanJ1939Header,
    pub pgn: u32,
}

#[derive(Clone, Copy)]
pub union CanProtoInfo {
    pub j1939: CanJ1939Info,
    pub idotp: CanIsoTpInfo,
}

#[derive(Clone, Copy)]
pub struct CanRecvInfo {
    pub proto: CanProtoInfo,
    pub stamp: u64,
    pub count: isize,
    pub iface: i32,
}

pub struct SockCanHandle {
    pub sockfd: ::std::os::raw::c_int,
    pub mode: SockCanMod,
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
                cglue::can_SOCK_x_RAW as i32,
                cglue::can_SOCK_x_CANRAW as i32,
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

    pub fn close(&self) {
        unsafe { cglue::close(self.sockfd) };
    }

    pub fn as_rawfd(&self) -> i32 {
        self.sockfd
    }

    pub fn get_ifname(&self, iface: i32) -> Result<String, CanError> {
        let mut ifreq: cglue::ifreq = unsafe { mem::MaybeUninit::uninit().assume_init() };
        ifreq.ifr_ifru.ifru_ivalue /* ifr_index */= iface;
        let rc = unsafe { cglue::ioctl(self.sockfd, cglue::can_SOCK_x_SIOCGIFNAME as u64, &ifreq) };

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

    pub fn set_timestamp(&mut self, timestamp: CanTimeStamp) -> Result<&mut Self, CanError> {
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

    pub fn recv_can_msg(&self, buffer: *mut u8, size: u32) -> CanRecvInfo {
        let mut info: CanRecvInfo = unsafe { mem::zeroed::<CanRecvInfo>() };

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

        info.count =
            unsafe { cglue::recvmsg(self.sockfd, &msg_hdr as *const _ as *mut cglue::msghdr, 0) };

        if msg_hdr.msg_namelen >= 8 {
            let canaddr = unsafe { &*(msg_hdr.msg_name as *const _ as *mut cglue::sockaddr_can) };
            info.iface= canaddr.can_ifindex
        }

        // ref: https://github.com/torvalds/linux/blob/master/tools/testing/selftests/net/timestamping.c
        let mut cmsg = unsafe { &*cglue::CMSG_FIRSTHDR(&msg_hdr) };

        while cmsg as *const _ != 0 as *const _ {
            match cmsg.cmsg_level as u32 {
                cglue::can_SOCK_x_SOL_SOCKET => match cmsg.cmsg_type as u32 {
                    cglue::can_RAW_x_SO_TIMESTAMPING => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timespec)
                        };
                        info.stamp = (time.tv_sec * 1000000 + time.tv_nsec) as u64;
                        break;
                    }
                    cglue::can_RAW_x_SO_TIMESTAMP_NEW => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timespec)
                        };
                        info.stamp = (time.tv_sec * 1000000 + time.tv_nsec) as u64;
                        break;
                    }
                    cglue::can_RAW_x_SO_TIMESTAMPNS => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timespec)
                        };
                        info.stamp = (time.tv_sec * 1000000 + time.tv_nsec) as u64;
                        break;
                    }
                    cglue::can_RAW_x_SO_TIMESTAMP => {
                        let time = unsafe {
                            &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut cglue::timeval)
                        };
                        info.stamp = (time.tv_sec * 1000000 + time.tv_usec) as u64;
                        break;
                    }
                    _ => {}
                },

                cglue::can_J1939_x_SOL_CAN_J1939 => {
                    info.iface = canaddr.can_ifindex;
                    info.proto.j1939.src.name = unsafe { canaddr.can_addr.j1939.name };
                    info.proto.j1939.src.addr = unsafe { canaddr.can_addr.j1939.addr };
                    info.proto.j1939.pgn = unsafe { canaddr.can_addr.j1939.pgn };

                    match cmsg.cmsg_type as u32 {
                        cglue::can_J1939_x_SCM_DEST_ADDR => {
                            let addr = unsafe { &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut u8) };
                            info.proto.j1939.dst.addr = *addr;
                        }
                        cglue::can_J1939_x_SCM_DEST_NAME => {
                            let name =
                                unsafe { &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut u64) };
                            info.proto.j1939.dst.name = *name;
                        }
                        // cglue::can_J1939_x_SCM_PRIO => {
                        //     let prio = unsafe { &*(cglue::CMSG_DATA(cmsg) as *const _ as *mut u8) };
                        //     info.j1939.prio= *prio;
                        // }
                        _ => {}
                    }
                }
                _ => {}
            }
            cmsg = unsafe { &*cglue::CMSG_NXTHDR(&msg_hdr, cmsg) };
        }
        info
    }

    pub fn get_can_frame(&self) -> SockCanMsg {
        #[allow(invalid_value)]
        let mut buffer: [u8; cglue::can_MTU_x_FD_MTU as usize] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let info = self.recv_can_msg(buffer.as_mut_ptr(), cglue::can_MTU_x_FD_MTU);

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
