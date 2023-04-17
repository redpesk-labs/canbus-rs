/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * $RP_BEGIN_LICENSE$
 * Commercial License Usage
 *  Licensees holding valid commercial IoT.bzh licenses may use this file in
 *  accordance with the commercial license agreement provided with the
 *  Software or, alternatively, in accordance with the terms contained in
 *  a written agreement between you and The IoT.bzh Company. For licensing terms
 *  and conditions see https://www.iot.bzh/terms-conditions. For further
 *  information use the contact form at https://www.iot.bzh/contact.
 *
 * GNU General Public License Usage
 *  Alternatively, this file may be used under the terms of the GNU General
 *  Public license version 3. This license is as published by the Free Software
 *  Foundation and appearing in the file LICENSE.GPLv3 included in the packaging
 *  of this file. Please review the following information to ensure the GNU
 *  General Public License requirements will be met
 *  https://www.gnu.org/licenses/gpl-3.0.html.
 * $RP_END_LICENSE$
 */

#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("./capi/sockcan-map.rs");

use std::ffi::{CStr};
use std::ffi::CString;
use std::mem;
const MAX_ERROR_LEN: usize = 256;

pub fn get_perror() -> String {
    let mut buffer = [0 as ::std::os::raw::c_char; MAX_ERROR_LEN];
    let cerror = unsafe { strerror_r(*__errno_location(), &mut buffer as *mut i8, MAX_ERROR_LEN) };
    let cstring = unsafe { CStr::from_ptr(cerror) };
    let slice: &str = cstring.to_str().unwrap();
    slice.to_owned()
}

// reference https://github.com/rust-lang/libc/blob/master/src/unix/linux_like/mod.rs
pub const fn CMSG_ALIGN(len: usize) -> usize {
    len + mem::size_of::<usize>() - 1 & !(mem::size_of::<usize>() - 1)
}

pub fn CMSG_NXTHDR(mhdr: *const msghdr, cmsg: *const cmsghdr) -> *mut cmsghdr {
    if (unsafe { (*cmsg).cmsg_len as usize }) < mem::size_of::<cmsghdr>() {
        return 0 as *mut cmsghdr;
    };
    let next = (cmsg as usize + CMSG_ALIGN(unsafe { (*cmsg).cmsg_len as usize })) as *mut cmsghdr;
    let max = unsafe { (*mhdr).msg_control as usize } + unsafe { (*mhdr).msg_controllen as usize };
    if (unsafe { next.offset(1) }) as usize > max
        || next as usize + CMSG_ALIGN(unsafe { (*next).cmsg_len as usize }) > max
    {
        0 as *mut cmsghdr
    } else {
        next as *mut cmsghdr
    }
}

pub fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    unsafe {
        if (*mhdr).msg_controllen as usize >= mem::size_of::<cmsghdr>() {
            (*mhdr).msg_control as *mut cmsghdr
        } else {
            0 as *mut cmsghdr
        }
    }
}

pub fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut std::ffi::c_uchar {
    unsafe { cmsg.offset(1) as *mut std::ffi::c_uchar }
}

// pub {const} fn CMSG_SPACE(length: std::ffi::c_uint) -> std::ffi::c_uint {
//     (CMSG_ALIGN(length as usize) + CMSG_ALIGN(mem::size_of::<cmsghdr>()))
//         as std::ffi::c_uint
// }

pub fn CMSG_LEN(length: std::ffi::c_uint) -> std::ffi::c_uint {
    CMSG_ALIGN(mem::size_of::<cmsghdr>()) as std::ffi::c_uint + length
}

// return Linux current date/time as a string
pub fn get_time(format: &str) -> Result<String,()> {
    let fmt= match CString::new(format) {
        Err(_err) => return Err(()),
        Ok(value) => value,
    };
    let time= unsafe {time (0 as *mut time_t)};
    let locale= unsafe{ localtime(&time)};
    let mut buffer= [0_i8;64];
    unsafe {strftime(buffer.as_mut_ptr(), buffer.len(), fmt.as_ptr(),locale)};
    let cstring = unsafe {CStr::from_ptr(buffer.as_ptr())};
    let slice= match cstring.to_str() {
        Err(_err) => return Err(()),
        Ok(value) => value,
    };
    Ok(slice.to_owned())
}