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

use super::cglue;
use crate::prelude::*;
use std::mem::{self, MaybeUninit};

bitflags! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct CanBcmFlag: u32 {
        const SET_TIMER         = cglue::can_BCM_FLAG_x_SETTIMER;
        const START_TIMER       = cglue::can_BCM_FLAG_x_STARTTIMER;
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
        const NONE =0;
    }
}
impl CanBcmFlag {
    pub fn check(flags: CanBcmFlag, value: u32) -> bool {
        flags.bits() & value != 0
    }
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

struct CanBcmOneMsg(cglue::can_bcm_one_msg);
struct CanFdBcmOneMsg(cglue::canfd_bcm_one_msg);
struct CanBcmHeader(cglue::bcm_msg_head);

pub struct SockBcmMsg {
    opcode: CanBcmOpCode,
    info: CanRecvInfo,
    frame: CanAnyFrame,
}

impl SockBcmMsg {
    pub fn get_iface(&self) -> i32 {
        self.info.iface
    }

    pub fn get_opcode(&self) -> CanBcmOpCode {
        self.opcode
    }

    pub fn get_stamp(&self) -> u64 {
        self.info.stamp
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

pub trait SockCanBmc {
    fn open_bcm<T>(candev: T, timestamp: CanTimeStamp) -> Result<SockCanHandle, CanError>
    where
        SockCanHandle: CanIFaceFrom<T>;
    fn get_bcm_frame(&self) -> SockBcmMsg;
}

impl SockCanBmc for SockCanHandle {
    fn open_bcm<T>(candev: T, timestamp: CanTimeStamp) -> Result<SockCanHandle, CanError>
    where
        SockCanHandle: CanIFaceFrom<T>,
    {
        let sockfd = unsafe {
            cglue::socket(
                cglue::can_SOCK_x_AF_CAN as i32,
                cglue::can_SOCK_x_DGRAM as i32,
                cglue::can_SOCK_x_BCM as i32,
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

    fn get_bcm_frame(&self) -> SockBcmMsg {
        #[allow(invalid_value)]
        let mut buffer: [u8; mem::size_of::<CanFdBcmOneMsg>()] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let info = self.recv_can_msg(buffer.as_mut_ptr(), mem::size_of::<CanFdBcmOneMsg>() as u32);
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
            info: info,
        }
    }
}

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

            CanBcmOpCode::RxDelete => {
                if self.frames.len() != 0 {
                    return Err(CanError::new(
                        "invalid-socketcan-filter",
                        "BCM:RxDelete does not accept frames",
                    ));
                };
            }
            _ => {
                return Err(CanError::new(
                    "invalid-bcm-operation",
                    "bcm operation not yet implemented",
                ));
            }
        }

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
