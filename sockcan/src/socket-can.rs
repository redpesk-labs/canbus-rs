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
use std::cell::RefCell;
use std::ffi::CStr;
use std::os::raw::c_char;

use super::cglue;
use crate::prelude::*;
use std::mem::{self};

pub type SockCanId = cglue::canid_t;

// Flag/masks — align with your bindings/constants
const CAN_EFF_FLAG: u32 = 0x8000_0000;
const CAN_SFF_MASK: u32 = 0x7FF;
const CAN_EFF_MASK: u32 = 0x1FFF_FFFF;

/// Normalize a user-provided CAN identifier:
/// - if `id <= 0x7FF`, return 11-bit Standard ID (no flags)
/// - else, return 29-bit Extended ID with CAN_EFF_FLAG set
#[inline]
fn normalize_can_id(id: u32) -> u32 {
    if id <= CAN_SFF_MASK {
        id & CAN_SFF_MASK
    } else {
        (id & CAN_EFF_MASK) | CAN_EFF_FLAG
    }
}

/// Build a Classical CAN frame (0..=8 bytes)
#[inline]
fn build_std_frame(id: u32, data: &[u8]) -> Result<cglue::can_frame, CanError> {
    if data.len() > 8 {
        return Err(CanError::new("can-build-std", "payload > 8 bytes; use FD"));
    }

    // Zero-init then fill only the relevant fields.
    let mut f: cglue::can_frame = unsafe { mem::zeroed() };
    f.can_id = normalize_can_id(id);
    // Classical CAN length (DLC) is stored in the anonymous union field `len`
    f.__bindgen_anon_1.len = u8::try_from(data.len()).unwrap_or(8);
    // Copy payload (remaining bytes already zeroed)
    for (i, b) in data.iter().enumerate() {
        f.data[i] = *b;
    }
    Ok(f)
}

/// Build a CAN FD frame (0..=64 bytes)
#[inline]
fn build_fd_frame(id: u32, data: &[u8], flags: u8) -> Result<cglue::canfd_frame, CanError> {
    if data.len() > 64 {
        return Err(CanError::new("can-build-fd", "payload > 64 bytes"));
    }
    let mut f: cglue::canfd_frame = unsafe { mem::zeroed() };
    f.can_id = normalize_can_id(id);
    f.len = u8::try_from(data.len()).unwrap_or(64);
    f.flags = flags; // e.g. BRS/ESI as needed by your app/driver
    for (i, b) in data.iter().enumerate() {
        f.data[i] = *b;
    }
    Ok(f)
}

/// User-facing helper that decides between Classical CAN and CAN FD.
///
/// *If `data.len() <= 8` → Classical; otherwise → FD (with `flags = 0`).*
pub fn generate_frame(id: u32, data: &[u8]) -> Result<CanAnyFrame, CanError> {
    if data.len() <= 8 {
        let f = build_std_frame(id, data)?;
        // Wrap in your raw type then into CanAnyFrame
        let raw = CanFrameRaw(f);
        Ok(CanAnyFrame::RawStd(raw))
    } else {
        let f = build_fd_frame(id, data, 0)?;
        let raw = CanFdFrameRaw(f);
        Ok(CanAnyFrame::RawFd(raw))
    }
}

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

#[derive(Debug, Copy, Clone)]
pub enum CanTimeStamp {
    NONE,
    CLASSIC,
    NANOSEC,
    HARDWARE,
    SOFTWARE,
}

/// Classical CAN frame structure (aka CAN 2.0B)
/// canid:   CAN ID of the frame and CAN_///_FLAG flags, see `canid_t` definition
/// @len:      CAN frame payload length in byte (0 .. 8)
/// @`can_dlc`:  deprecated name for CAN frame payload length in byte (0 .. 8)
/// @__pad:    padding
/// @__res0:   reserved / padding
/// @`len8_dlc`: optional DLC value (9 .. 15) at 8 byte payload length
///            `len8_dlc` contains values from 9 .. 15 when the payload length is
///            8 bytes but the DLC value (see ISO 11898-1) is greater then 8.
///            `CAN_CTRLMODE_CC_LEN8_DLC` flag has to be enabled in CAN driver.
/// @data:     CAN frame payload (up to 8 byte)
pub struct CanFrameRaw(pub cglue::can_frame);

impl CanFrameRaw {
    /// Constructs a classical CAN (2.0) frame wrapper from raw fields.
    ///
    /// This initializes an underlying `cglue::can_frame` with the given CAN
    /// identifier and an **explicit payload length** (`len`, 0..=8). The `data`
    /// array is always 8 bytes; only the first `len` bytes are considered part of
    /// the payload. The optional `len8_dlc` field is set to `0` (not used).
    ///
    /// > **Identifier note:** `canid` is taken **as-is** (kernel format).
    /// > If you want a 29-bit extended identifier, include the extended flag
    /// > (`CAN_EFF_FLAG`, i.e. `0x8000_0000`) yourself. For 11-bit standard IDs,
    /// > keep that flag cleared.
    ///
    /// # Parameters/// Constructs a classical CAN (2.0) frame wrapper from raw fields.
    ///
    /// This initializes an underlying `cglue::can_frame` with the given CAN
    /// identifier and an **explicit payload length** (`len`, 0..=8). The `data`
    /// array is always 8 bytes; only the first `len` bytes are considered part of
    /// the payload. The optional `len8_dlc` field is set to `0` (not used).
    ///
    /// > **Identifier note:** `canid` is taken **as-is** (kernel format).
    /// > If you want a 29-bit extended identifier, include the extended flag
    /// > (`CAN_EFF_FLAG`, i.e. `0x8000_0000`) yourself. For 11-bit standard IDs,
    /// > keep that flag cleared.
    ///
    /// # Parameters
    /// - `canid`: Raw CAN identifier with protocol flags (e.g. may include
    ///   `CAN_EFF_FLAG`, `CAN_RTR_FLAG`, `CAN_ERR_FLAG`) as expected by the kernel.
    /// - `len`: Payload length in bytes (`0..=8`). Caller is responsible for
    ///   providing a valid value consistent with `data`.
    /// - `pad`: Value for the kernel `__pad` field (usually `0`).
    /// - `res`: Value for the kernel reserved field `__res0` (usually `0`).
    /// - `data`: Fixed 8-byte buffer; only the first `len` bytes are treated as
    ///   payload.
    ///
    /// # Returns
    /// A `CanFrameRaw` wrapping a fully initialized `can_frame`.
    ///
    /// # Safety & invariants
    /// - No validation is performed: if `len > 8`, the behavior of downstream
    ///   consumers is undefined; pass a valid DLC (0..=8).
    /// - Flags inside `canid` are **not** normalized here; set/clear them upstream
    ///   as appropriate.
    ///
    /// # Example
    /// ```rust
    /// use crate::sockcan::CanFrameRaw;
    /// const CAN_EFF_FLAG: u32 = 0x8000_0000;
    ///
    /// // Standard ID 0x118, 8-byte payload
    /// let f_std = CanFrameRaw::new(0x118, 8, 0, 0, [0x59,0x06,0x22,0x18,0x00,0x00,0x00,0x00]);
    ///
    /// // Extended ID 0x1DF9050 (29-bit) with EFF flag set
    /// let ext_id = (0x01DF9050 & 0x1FFF_FFFF) | CAN_EFF_FLAG;
    /// let f_ext = CanFrameRaw::new(ext_id, 3, 0, 0, [0xDE,0xAD,0xBE,0,0,0,0,0]);
    /// ```
    /// - `canid`: Raw CAN identifier with protocol flags (e.g. may include
    ///   `CAN_EFF_FLAG`, `CAN_RTR_FLAG`, `CAN_ERR_FLAG`) as expected by the kernel.
    /// - `len`: Payload length in bytes (`0..=8`). Caller is responsible for
    ///   providing a valid value consistent with `data`.
    /// - `pad`: Value for the kernel `__pad` field (usually `0`).
    /// - `res`: Value for the kernel reserved field `__res0` (usually `0`).
    /// - `data`: Fixed 8-byte buffer; only the first `len` bytes are treated as
    ///   payload.
    ///
    /// # Returns
    /// A `CanFrameRaw` wrapping a fully initialized `can_frame`.
    ///
    /// # Safety & invariants
    /// - No validation is performed: if `len > 8`, the behavior of downstream
    ///   consumers is undefined; pass a valid DLC (0..=8).
    /// - Flags inside `canid` are **not** normalized here; set/clear them upstream
    ///   as appropriate.
    ///
    /// # Example
    /// ```rust
    /// use crate::sockcan::CanFrameRaw;
    /// const CAN_EFF_FLAG: u32 = 0x8000_0000;
    ///
    /// // Standard ID 0x118, 8-byte payload
    /// let f_std = CanFrameRaw::new(0x118, 8, 0, 0, [0x59,0x06,0x22,0x18,0x00,0x00,0x00,0x00]);
    ///
    /// // Extended ID 0x1DF9050 (29-bit) with EFF flag set
    /// let ext_id = (0x01DF9050 & 0x1FFF_FFFF) | CAN_EFF_FLAG;
    /// let f_ext = CanFrameRaw::new(ext_id, 3, 0, 0, [0xDE,0xAD,0xBE,0,0,0,0,0]);
    /// ```
    #[must_use]
    pub fn new(canid: SockCanId, len: u8, pad: u8, res: u8, data: [u8; 8usize]) -> Self {
        CanFrameRaw(cglue::can_frame {
            can_id: canid,
            __pad: pad,
            __res0: res,
            len8_dlc: 0,
            data,
            __bindgen_anon_1: cglue::can_frame__bindgen_ty_1 { len },
        })
    }
    #[must_use]
    pub fn empty(canid: u32) -> Self {
        let mut frame: CanFrameRaw = unsafe { mem::zeroed::<Self>() };
        frame.0.can_id = canid;
        frame
    }
    #[must_use]
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        (&raw const self.0).cast::<std::ffi::c_void>().cast_mut()
    }

    #[must_use]
    pub fn get_id(&self) -> SockCanId {
        (self.0.can_id as SockCanId) & !CAN_EFF_FLAG
    }

    #[must_use]
    pub fn get_len(&self) -> u8 {
        unsafe { self.0.__bindgen_anon_1.len }
    }

    #[must_use]
    pub fn get_data(&self) -> &[u8] {
        &self.0.data
    }
}
pub struct CanFdFrameRaw(pub cglue::canfd_frame);
impl CanFdFrameRaw {
    /// Constructs a **CAN FD** frame wrapper from raw fields.
    ///
    /// This initializes an underlying `cglue::canfd_frame` with the given CAN
    /// identifier, **payload length** (`len`, 0..=64), FD **flags**, and data
    /// buffer. The `data` array is always 64 bytes; only the first `len` bytes are
    /// considered part of the payload. Remaining bytes are kept as provided.
    ///
    /// > **Identifier note:** `canid` is taken **as-is** (kernel format).
    /// > For a 29-bit extended identifier, include the extended flag yourself
    /// > (e.g. `CAN_EFF_FLAG`, `0x8000_0000`). For an 11-bit standard ID, leave it cleared.
    ///
    /// > **Flags note (FD):** `flags` typically carries CAN FD controls such as:
    /// > - **BRS** (Bit Rate Switch), often `CANFD_BRS`
    /// > - **ESI** (Error State Indicator), often `CANFD_ESI`
    /// > Use the values exposed by your bindings (e.g., `cglue::CANFD_BRS`).
    ///
    /// # Parameters
    /// - `canid`: Raw CAN identifier with protocol flags (EFF/RTR/ERR as expected by the kernel).
    /// - `len`: Payload length in bytes (`0..=64`). Caller must provide a valid value
    ///   consistent with `data`.
    /// - `flags`: CAN FD flags (e.g., BRS/ESI).
    /// - `res0`, `res1`: Reserved fields passed to the kernel (usually `0`).
    /// - `data`: Fixed 64-byte buffer; only the first `len` bytes are treated as payload.
    ///
    /// # Returns
    /// A `CanFdFrameRaw` wrapping a fully initialized `canfd_frame`.
    ///
    /// # Safety & invariants
    /// - No validation is performed: if `len > 64`, downstream behavior is undefined.
    /// - `canid` flags are **not** normalized here; set/clear them upstream.
    /// - `flags` are **not** interpreted here; pass the appropriate constants for your use case.
    ///
    /// # Examples
    /// ```rust
    /// // Extended FD frame with BRS, 12-byte payload
    /// const CAN_EFF_FLAG: u32 = 0x8000_0000;
    /// let ext_id = (0x18DAF110 & 0x1FFF_FFFF) | CAN_EFF_FLAG;
    /// let data = [0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08, 0x09,0x0A,0x0B,0x0C,
    ///             0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
    ///             0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0];
    /// let flags = cglue::CANFD_BRS; // or 0 if no BRS/ESI
    /// let f_fd = CanFdFrameRaw::new(ext_id, 12, flags, 0, 0, data);
    ///
    /// // Standard FD frame (no EFF flag), 3-byte payload, no BRS
    /// let std_id = 0x123;
    /// let mut buf = [0u8; 64];
    /// buf[..3].copy_from_slice(&[0xDE, 0xAD, 0xBE]);
    /// let f_std_fd = CanFdFrameRaw::new(std_id, 3, 0, 0, 0, buf);
    /// ```
    #[must_use]
    pub fn new(
        canid: SockCanId,
        len: u8,
        flags: u8,
        res0: u8,
        res1: u8,
        data: [u8; 64usize],
    ) -> Self {
        CanFdFrameRaw(cglue::canfd_frame {
            can_id: canid,
            len,
            __res0: res0,
            __res1: res1,
            flags,
            data,
        })
    }

    #[must_use]
    pub fn empty(canid: u32) -> Self {
        let mut frame: CanFdFrameRaw = unsafe { mem::zeroed::<Self>() };
        frame.0.can_id = canid;
        frame
    }

    #[must_use]
    pub fn as_ptr(&self) -> *mut std::ffi::c_void {
        (&raw const self.0).cast::<std::ffi::c_void>().cast_mut()
    }

    #[must_use]
    pub fn get_id(&self) -> SockCanId {
        self.0.can_id as SockCanId
    }

    #[must_use]
    pub fn get_len(&self) -> u8 {
        self.0.len
    }

    #[must_use]
    pub fn get_flag(&self) -> u8 {
        self.0.flags
    }

    #[must_use]
    pub fn get_data(&self) -> &[u8] {
        &self.0.data
    }
}

/// A tagged container for any received CAN frame or a read outcome.
///
/// `CanAnyFrame` abstracts over **Classical CAN** and **CAN FD** raw frames and
/// can also represent an **I/O error** or a **non-data outcome** (e.g. timeout).
///
/// # Variants
/// - [`CanAnyFrame::RawStd`] — Classical CAN 2.0 frame (up to 8 bytes) backed by
///   [`CanFrameRaw`]. Its `get_id()` implementation returns the identifier with the
///   *extended* flag cleared (no `0x8000_0000`).
/// - [`CanAnyFrame::RawFd`] — CAN FD frame (up to 64 bytes) backed by
///   [`CanFdFrameRaw`]. Its `get_id()` returns the raw `can_id` as stored by the
///   kernel/driver. If you need a normalized 29-bit ID, mask with `0x1FFF_FFFF`.
/// - [`CanAnyFrame::Err`] — An error occurred while reading/decoding the frame
///   (e.g., short read, invalid size, OS error). `get_id()/get_len()/get_data()`
///   return this error.
/// - [`CanAnyFrame::None`] — No payload (e.g., timeout/announce). Carries an
///   optional `u32` identifier for context; length is reported as `0`, and data
///   is reported as a one-byte empty view.
///
/// # Accessors
/// Use the convenience methods to extract normalized values when available:
/// - [`CanAnyFrame::get_id`] → `Result<u32, CanError>`
/// - [`CanAnyFrame::get_len`] → `Result<u8, CanError>`
/// - [`CanAnyFrame::get_data`] → `Result<&[u8], CanError>`
///
/// These methods forward to the underlying raw type, or return the contained
/// error for [`Err`], or sensible defaults for [`None`] (ID/0, LEN/0, DATA/&[0]).
///
/// > **Note on identifiers:**
/// > For `RawStd`, the returned ID has the extended flag cleared. For `RawFd`,
/// > the ID is returned as-is; apply `id & 0x1FFF_FFFF` if you need the 29-bit
/// > value without protocol flags.
///
/// # Example
/// ```rust
/// match msg.get_raw() {
///     CanAnyFrame::RawStd(f) => {
///         let id = f.get_id();           // 11b or 29b (EFF cleared)
///         let dlc = f.get_len();
///         let data = f.get_data();
///         log::info!("STD {:08X} [{}] {:02X?}", id, dlc, &data[..dlc as usize]);
///     }
///     CanAnyFrame::RawFd(f) => {
///         let id = f.get_id() & 0x1FFF_FFFF; // normalize if needed
///         let len = f.get_len();
///         let data = f.get_data();
///         log::info!("FD  {:08X} [{}] {:02X?}", id, len, &data[..len as usize]);
///     }
///     CanAnyFrame::Err(e) => {
///         log::error!("CAN read error: {e}");
///     }
///     CanAnyFrame::None(canid) => {
///         log::debug!("timeout/announce for id {:08X}", canid);
///     }
/// }
/// ```
pub enum CanAnyFrame {
    /// Classical CAN frame (CAN 2.0B).
    RawFd(CanFdFrameRaw),
    /// CAN FD frame (up to 64 bytes).
    RawStd(CanFrameRaw),
    /// I/O or decode error while fetching a frame.
    Err(CanError),
    /// No data (e.g., timeout/announce), with an optional CAN ID context.
    None(u32),
}

impl CanAnyFrame {
    /// Returns id.
    ///
    /// # Errors
    /// Returns `CanError` if id is unavailable.
    pub fn get_id(&self) -> Result<u32, CanError> {
        match self {
            CanAnyFrame::RawFd(frame) => Ok(frame.get_id()),
            CanAnyFrame::RawStd(frame) => Ok(frame.get_id()),
            CanAnyFrame::Err(error) => Err(error.clone()),
            CanAnyFrame::None(canid) => Ok(*canid),
        }
    }

    /// Returns len.
    ///
    /// # Errors
    /// Returns `CanError` if len is unavailable.
    pub fn get_len(&self) -> Result<u8, CanError> {
        match self {
            CanAnyFrame::RawFd(frame) => Ok(frame.get_len()),
            CanAnyFrame::RawStd(frame) => Ok(frame.get_len()),
            CanAnyFrame::Err(error) => Err(error.clone()),
            CanAnyFrame::None(_canid) => Ok(0),
        }
    }
    /// Returns data.
    ///
    /// # Errors
    /// Returns `CanError` if data is unavailable.
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
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(src: *const CanFrameRaw) -> Self {
        let dst = unsafe {
            debug_assert!(!src.is_null(), "null CanFrameRaw pointer");

            let mut tmp = core::mem::MaybeUninit::<CanFrameRaw>::uninit();
            core::ptr::copy_nonoverlapping(src, tmp.as_mut_ptr(), 1);
            tmp.assume_init()
        };
        CanAnyFrame::RawStd(dst)
    }
}

impl From<*const CanFdFrameRaw> for CanAnyFrame {
    // La signature de `From::from` ne peut pas être `unsafe`, on documente
    // et on désactive localement l’avertissement Clippy.
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(src: *const CanFdFrameRaw) -> Self {
        let dst = unsafe {
            debug_assert!(!src.is_null(), "null CanFdFrameRaw pointer");

            let mut tmp = core::mem::MaybeUninit::<CanFdFrameRaw>::uninit();
            core::ptr::copy_nonoverlapping(src, tmp.as_mut_ptr(), 1);
            tmp.assume_init()
        };
        CanAnyFrame::RawFd(dst)
    }
}

pub struct SockCanMsg {
    iface: i32,
    stamp: u64,
    frame: CanAnyFrame,
}

impl SockCanMsg {
    #[must_use]
    pub fn get_iface(&self) -> i32 {
        self.iface
    }

    #[must_use]
    pub fn get_stamp(&self) -> u64 {
        self.stamp
    }

    #[must_use]
    pub fn get_raw(&self) -> &CanAnyFrame {
        &self.frame
    }
    /// Returns the payload length (DLC) of this CAN frame.
    ///
    /// The value reflects the number of data bytes present:
    /// - Classical CAN: typically `0..=8`
    /// - CAN FD: up to `64`, depending on flags and DLC encoding
    ///
    /// # Returns
    /// The payload length as `u8`.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the frame is malformed or not fully initialized;
    /// - the reported DLC exceeds the available buffer length (truncated frame);
    /// - the DLC is invalid for the frame type (e.g., > 8 for Classical CAN without FD);
    /// - frame flags (e.g., FD/RTR) are inconsistent with the stored length.
    pub fn get_len(&self) -> Result<u8, CanError> {
        self.frame.get_len()
    }
    /// Returns the CAN identifier (11-bit standard or 29-bit extended) for this frame.
    ///
    /// The value is returned as a `u32` with the identifier bits already normalized
    /// (i.e., without protocol flag bits).
    ///
    /// # Returns
    /// The extracted CAN identifier as `u32`.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the frame header is missing or malformed (e.g., not enough bytes to read the ID);
    /// - the identifier bits are inconsistent with the frame flags (e.g., EFF/RTR/ERR);
    /// - the computed identifier exceeds the valid bit-width for the reported frame type;
    /// - internal validation of the underlying buffer/length fails.
    pub fn get_id(&self) -> Result<u32, CanError> {
        self.frame.get_id()
    }
    /// Returns a borrowed view of the CAN frame payload.
    ///
    /// For classic CAN this slice is at most 8 bytes; for CAN FD it can be larger
    /// (up to 64 bytes), depending on the underlying frame.
    ///
    /// # Returns
    /// A borrowed byte slice referencing the payload contained in this frame.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the internal raw frame length is invalid or inconsistent with the frame type;
    /// - the payload pointer/offset computed from the raw frame is out of bounds;
    /// - the frame is not initialized (e.g., was built from an incomplete OS read);
    /// - any invariant required to expose a safe slice is violated.
    ///
    /// On success, the returned slice is tied to `&self` and does not allocate.
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

#[derive(Clone)]
pub enum CanProtoInfo {
    J1939(CanJ1939Info),
    IsoTp(CanIsoTpInfo),
    Error(CanError),
    Retry,
    None,
}

#[derive(Clone)]
pub struct CanRecvInfo {
    pub proto: CanProtoInfo,
    pub stamp: u64,
    pub count: isize,
    pub iface: i32,
}

#[derive(Clone)]
pub enum SockCanOpCode {
    RxRead(Vec<u8>),
    RxError(CanError),
    RxPartial(u8),
    RxIgnore,
    RxInvalid,
}
pub trait SockCanCtrl {
    fn check_frame(&self, data: &[u8], info: &CanRecvInfo) -> SockCanOpCode;
}

pub struct SockCanHandle {
    pub sockfd: ::std::os::raw::c_int,
    pub mode: SockCanMod,
    pub callback: Option<RefCell<Box<dyn SockCanCtrl>>>,
}

pub trait CanIFaceFrom<T> {
    fn map_can_iface(sock: i32, iface: T) -> i32;
}

impl CanIFaceFrom<&str> for SockCanHandle {
    /// Resolves a CAN interface name (e.g. `"can0"`, `"vcan0"`) into its kernel index.
    ///
    /// This fills an `ifreq` structure with the provided interface name and issues
    /// `ioctl(SIOCGIFINDEX)` to retrieve the interface index.
    ///
    /// * The name is **truncated** to `IFACE_LEN-1` bytes and is always **NUL-terminated**.
    /// * Any interior `'\0'` byte in `iface` acts as an early terminator (defensive).
    ///
    /// # Parameters
    /// - `sock`: an open CAN socket fd (used only for the ioctl call).
    /// - `iface`: interface name (`"can0"`, `"vcan0"`, …).
    ///
    /// # Returns
    /// - `< 0` on failure (kernel error code from `ioctl`),
    /// - interface index (`ifindex`) on success.
    fn map_can_iface(sock: i32, iface: &str) -> i32 {
        // Zero-init ensures NUL-termination of the name buffer.
        let mut ifreq: cglue::ifreq = unsafe { mem::zeroed() };

        // Copy at most (LEN-1) bytes and stop early if a NUL is present in `iface`.
        let iname = iface.as_bytes();

        for (idx, &b) in iname.iter().take(cglue::can_SOCK_x_IFACE_LEN as usize).enumerate() {
            // Évite cast_possible_wrap: on borne aux valeurs ASCII sûres
            let ch: c_char = match i8::try_from(b) {
                Ok(v) => v as c_char,
                Err(_) => 0 as c_char, // ou `continue` si tu préfères ignorer
            };

            unsafe {
                ifreq.ifr_ifrn.ifrn_name[idx] = ch;
            }
        }

        // Query the interface index
        let rc = unsafe { cglue::ioctl(sock, u64::from(cglue::can_SOCK_x_SIOCGIFINDEX), &ifreq) };

        if rc < 0 {
            rc
        } else {
            // ifr_ifindex
            unsafe { ifreq.ifr_ifru.ifru_ivalue } //ifr.ifr_if index
        }
    }
}

impl CanIFaceFrom<u32> for SockCanHandle {
    fn map_can_iface(_sock: i32, iface: u32) -> i32 {
        i32::try_from(iface).unwrap_or(i32::MAX)
    }
}

impl SockCanHandle {
    /// Opens a RAW CAN socket on the specified CAN interface.
    ///
    /// This creates and configures a RAW CAN socket (`PF_CAN/RAW`), optionally
    /// enabling timestamping according to `timestamp`, and binds it to the
    /// provided interface.
    ///
    /// # Type Parameters
    /// - `T`: Any type that can be converted into an interface handle via
    ///   `SockCanHandle: CanIFaceFrom<T>` (e.g., interface index, name, or wrapper).
    ///
    /// # Parameters
    /// - `candev`: The CAN interface identifier (e.g., `"can0"` or an index).
    /// - `timestamp`: Timestamping mode to apply to the socket (e.g., disabled,
    ///   software, or hardware).
    ///
    /// # Returns
    /// A newly opened `SockCanHandle` wrapped in `Self` on success.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the socket cannot be created (e.g., `socket(PF_CAN, SOCK_RAW, CAN_RAW)` fails);
    /// - the interface cannot be resolved from `candev` (invalid name/index or missing device);
    /// - applying socket options for the requested `timestamp` mode fails;
    /// - binding the socket to the interface fails (e.g., insufficient permissions or
    ///   interface is down);
    /// - any unexpected OS error occurs during setup (e.g., `fcntl`, `setsockopt`, `bind`).
    pub fn open_raw<T>(candev: T, timestamp: CanTimeStamp) -> Result<Self, CanError>
    where
        SockCanHandle: CanIFaceFrom<T>,
    {
        let pf_can = i32::try_from(cglue::can_SOCK_x_PF_CAN).unwrap_or(i32::MAX);
        let raw_ty = i32::try_from(cglue::can_SOCK_x_RAW).unwrap_or(i32::MAX);
        let proto = i32::try_from(cglue::can_SOCK_x_CANRAW).unwrap_or(i32::MAX);

        let sockfd = unsafe { cglue::socket(pf_can, raw_ty, proto) };

        if sockfd < 0 {
            return Err(CanError::new("fail-socketcan-open", cglue::get_perror()));
        }

        let index = SockCanHandle::map_can_iface(sockfd, candev);
        if index < 0 {
            return Err(CanError::new("fail-socketcan-iface", cglue::get_perror()));
        }

        #[allow(invalid_value)]
        let mut canaddr: cglue::sockaddr_can = unsafe { std::mem::zeroed() };

        let fam = u16::try_from(cglue::can_SOCK_x_PF_CAN).map_err(CanError::from)?;
        canaddr.can_family = fam;

        canaddr.can_ifindex = index;

        let sockaddr = cglue::__CONST_SOCKADDR_ARG {
            __sockaddr__: (&raw const canaddr).cast::<cglue::sockaddr>(),
        };
        let socklen = cglue::socklen_t::try_from(mem::size_of::<cglue::sockaddr_can>())
            .map_err(CanError::from)?;
        let status = unsafe { cglue::bind(sockfd, sockaddr, socklen) };
        if status < 0 {
            return Err(CanError::new("fail-socketcan-bind", cglue::get_perror()));
        }

        let mut sockcan = SockCanHandle { mode: SockCanMod::RAW, sockfd, callback: None };

        match sockcan.set_timestamp(timestamp) {
            Err(error) => return Err(error),
            Ok(_value) => {},
        }

        Ok(sockcan)
    }

    pub fn set_callback(&mut self, callback: Box<dyn SockCanCtrl>) {
        self.callback = Some(RefCell::new(callback));
    }

    pub fn close(&self) {
        unsafe { cglue::close(self.sockfd) };
    }

    pub fn as_rawfd(&self) -> i32 {
        self.sockfd
    }
    /// Returns the network interface name (e.g., `"can0"`) for a given interface index.
    ///
    /// Internally this queries the kernel (e.g., via `ioctl(SIOCGIFNAME)` or an
    /// equivalent mechanism) and converts the returned C string to `String`.
    ///
    /// # Parameters
    /// - `iface`: The interface index (as returned by `SIOCGIFINDEX`, `if_nametoindex`,
    ///   or other enumeration of CAN interfaces).
    ///
    /// # Returns
    /// The interface name as a `String` on success.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the OS call to resolve the index to a name fails (invalid index, insufficient
    ///   privileges, or kernel/driver limitation);
    /// - the returned name is not valid UTF-8 and cannot be converted to `String`;
    /// - an ABI mismatch occurs (unexpected struct layout/size) causing the query to fail;
    /// - any other unexpected OS error is reported while fetching the interface name.
    pub fn get_ifname(&self, iface: i32) -> Result<String, CanError> {
        let mut ifreq: cglue::ifreq = unsafe { mem::MaybeUninit::uninit().assume_init() };
        ifreq.ifr_ifru.ifru_ivalue /* ifr_index */= iface;
        let rc =
            unsafe { cglue::ioctl(self.sockfd, u64::from(cglue::can_SOCK_x_SIOCGIFNAME), &ifreq) };

        if rc < 0 {
            Err(CanError::new("can-ifname-fail", cglue::get_perror()))
        } else {
            let name_ptr = unsafe {
                core::ptr::addr_of!(ifreq.ifr_ifrn.ifrn_name).cast::<std::os::raw::c_char>()
            };
            let cstring = unsafe { CStr::from_ptr(name_ptr) };
            match cstring.to_str() {
                Err(error) => Err(CanError::new("can-ifname-invalid", error.to_string())),
                Ok(slice) => Ok(slice.to_owned()),
            }
        }
    }
    /// Enables or disables non-blocking mode on the underlying CAN socket.
    ///
    /// When non-blocking mode is disabled (`blocking = true`), I/O operations
    /// may block until they can complete. When enabled (`blocking = false`),
    /// operations return immediately with `EWOULDBLOCK`/`EAGAIN` if they would
    /// otherwise block.
    ///
    /// # Parameters
    /// - `blocking`: `true` to use blocking I/O, `false` for non-blocking.
    ///
    /// # Returns
    /// A mutable reference to `self` on success (for call chaining).
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - fetching current file status flags via `fcntl(F_GETFL)` fails;
    /// - updating flags with `fcntl(F_SETFL)` fails (e.g., invalid descriptor,
    ///   insufficient permissions, or platform-specific limitations);
    /// - the socket handle is invalid or not a CAN socket;
    /// - an unexpected OS error occurs while toggling non-blocking mode.
    pub fn set_blocking(&mut self, blocking: bool) -> Result<&mut Self, CanError> {
        // retrieve current flags
        let f_getfl: i32 = i32::try_from(cglue::can_SOCK_x_F_GETFL).unwrap_or(i32::MAX);
        let current_flag = unsafe { cglue::fcntl(self.sockfd, f_getfl) };
        if current_flag < 0 {
            return Err(CanError::new("can-nonblock-fail", cglue::get_perror()));
        }

        let nb: i32 = i32::try_from(cglue::can_SOCK_x_NONBLOCK).unwrap_or(i32::MAX);
        let new_flag = if blocking { current_flag & !nb } else { current_flag | nb };

        let f_set_fl: i32 = i32::try_from(cglue::can_SOCK_x_F_SETFL).unwrap_or(i32::MAX);
        let status = unsafe { cglue::fcntl(self.sockfd, f_set_fl, new_flag) };
        if status < 0 {
            return Err(CanError::new("can-nonblock-fail", cglue::get_perror()));
        }
        Ok(self)
    }
    /// Sets read and write timeouts on the RAW CAN socket.
    ///
    /// Configures `SO_RCVTIMEO` and `SO_SNDTIMEO` using millisecond values.
    /// A value of `0` typically means “no timeout” (blocking), while a positive
    /// value sets the maximum blocking duration for the corresponding operation.
    ///
    /// # Parameters
    /// - `read_ms`: Receive timeout in milliseconds for `recv/recvmsg`.
    /// - `write_ms`: Send timeout in milliseconds for `send/sendmsg`.
    ///
    /// # Returns
    /// A mutable reference to `self` on success (to allow call chaining).
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - converting the millisecond values to `timeval` overflows or is invalid;
    /// - a `setsockopt(SO_RCVTIMEO|SO_SNDTIMEO)` call fails (e.g., due to
    ///   insufficient privileges, invalid arguments, or kernel/driver limitations);
    /// - the socket handle is invalid or not a CAN RAW socket;
    /// - the kernel rejects the option value or size (ABI mismatch).
    pub fn set_timeout(&mut self, read_ms: i64, write_ms: i64) -> Result<&mut Self, CanError> {
        if read_ms > 0 {
            let timout =
                cglue::timeval { tv_sec: read_ms / 1000, tv_usec: read_ms * 1000 % 1_000_000 };
            unsafe {
                let status = cglue::setsockopt(
                    self.sockfd,
                    i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX),
                    i32::try_from(cglue::can_SOCK_x_SO_RCVTIMEO).unwrap_or(i32::MAX),
                    core::ptr::addr_of!(timout).cast::<::std::os::raw::c_void>(),
                    cglue::socklen_t::try_from(mem::size_of::<cglue::timeval>())
                        .unwrap_or(u32::MAX),
                );

                if status < 0 {
                    return Err(CanError::new("can-read-fail", cglue::get_perror()));
                }
            }
        }

        if write_ms > 0 {
            let timout =
                cglue::timeval { tv_sec: write_ms / 1000, tv_usec: write_ms * 1000 % 1_000_000 };
            unsafe {
                let status = cglue::setsockopt(
                    self.sockfd,
                    i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX),
                    i32::try_from(cglue::can_SOCK_x_SO_SNDTIMEO).unwrap_or(i32::MAX),
                    core::ptr::addr_of!(timout).cast::<::std::os::raw::c_void>(),
                    cglue::socklen_t::try_from(mem::size_of::<cglue::timeval>())
                        .unwrap_or(u32::MAX),
                );

                if status < 0 {
                    return Err(CanError::new("can-read-fail", cglue::get_perror()));
                }
            }
        }

        Ok(self)
    }
    /// Enables or disables CAN loopback on the RAW socket.
    ///
    /// When loopback is enabled, frames sent by this socket can be received back
    /// on the same socket (useful for testing). Disabling loopback suppresses this
    /// behavior.
    ///
    /// # Parameters
    /// - `loopback`: `true` to enable loopback; `false` to disable it.
    ///
    /// # Returns
    /// A mutable reference to `self` on success (for call chaining).
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the underlying `setsockopt` call fails (e.g., due to insufficient
    ///   privileges, invalid value, or missing kernel support);
    /// - the socket handle is invalid or not a CAN RAW socket;
    /// - the option value/size does not match what the kernel expects.
    pub fn set_loopback(&mut self, loopback: bool) -> Result<&mut Self, CanError> {
        let flag = i32::from(loopback);
        let status = unsafe {
            cglue::setsockopt(
                self.sockfd,
                i32::try_from(cglue::can_RAW_x_SOL_CAN_RAW).unwrap_or(i32::MAX),
                i32::try_from(cglue::can_RAW_x_RECV_OWN_MSGS).unwrap_or(i32::MAX),
                core::ptr::addr_of!(flag).cast::<std::ffi::c_void>(),
                cglue::socklen_t::try_from(mem::size_of::<i32>()).unwrap_or(u32::MAX),
            )
        };
        if status < 0 {
            return Err(CanError::new("can-loopback-fail", cglue::get_perror()));
        }
        Ok(self)
    }
    /// Enables kernel timestamping on the RAW CAN socket.
    ///
    /// Depending on `timestamp`, this configures which timestamping mode the socket
    /// should use (e.g., software timestamps, `so_timestampns`, `SO_TIMESTAMPING`).
    ///
    /// # Parameters
    /// - `timestamp`: Desired timestamping policy (see `CanTimeStamp`).
    ///
    /// # Returns
    /// A mutable reference to `self` on success (for call chaining).
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the underlying `setsockopt` call fails (e.g., invalid arguments,
    ///   insufficient privileges, or kernel does not support the requested mode);
    /// - the option value/size does not match what the kernel expects;
    /// - the socket handle is invalid or not a CAN RAW socket.
    pub fn set_timestamp(&mut self, timestamp: CanTimeStamp) -> Result<&mut Self, CanError> {
        let status = match timestamp {
            CanTimeStamp::SOFTWARE => {
                let flag = cglue::can_RAW_x_SOF_TIMESTAMPING_RX_SOFTWARE;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX),
                        i32::try_from(cglue::can_RAW_x_SO_TIMESTAMPING).unwrap_or(i32::MAX),
                        (&raw const flag).cast::<std::ffi::c_void>(),
                        cglue::socklen_t::try_from(mem::size_of::<i32>()).unwrap_or(u32::MAX),
                    )
                }
            },
            CanTimeStamp::HARDWARE => {
                let flag = cglue::can_RAW_x_SOF_TIMESTAMPING_RX_HARDWARE;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX),
                        i32::try_from(cglue::can_RAW_x_SO_TIMESTAMPING).unwrap_or(i32::MAX),
                        (&raw const flag).cast::<std::ffi::c_void>(),
                        cglue::socklen_t::try_from(mem::size_of::<i32>()).unwrap_or(u32::MAX),
                    )
                }
            },
            CanTimeStamp::NANOSEC => {
                let flag = 1_u32;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX),
                        i32::try_from(cglue::can_RAW_x_SO_TIMESTAMPNS).unwrap_or(i32::MAX),
                        (&raw const flag).cast::<std::ffi::c_void>(),
                        cglue::socklen_t::try_from(mem::size_of::<i32>()).unwrap_or(u32::MAX),
                    )
                }
            },
            CanTimeStamp::CLASSIC => {
                let flag: u32 = 1;
                unsafe {
                    cglue::setsockopt(
                        self.sockfd,
                        i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX),
                        i32::try_from(cglue::can_RAW_x_SO_TIMESTAMP).unwrap_or(i32::MAX),
                        (&raw const flag).cast::<std::ffi::c_void>(),
                        cglue::socklen_t::try_from(core::mem::size_of::<u32>()).unwrap_or(u32::MAX),
                    )
                }
            },
            CanTimeStamp::NONE => 0,
        };

        if status < 0 {
            return Err(CanError::new("can-setsock-stamp-fail", cglue::get_perror()));
        }
        Ok(self)
    }
    /// Enables kernel error monitoring on the RAW CAN socket using `mask`.
    ///
    /// Sets the CAN error-mask so that error frames matching `mask` are reported
    /// by the socket (e.g., bus-off, error-passive, tx-timeout).
    ///
    /// # Parameters
    /// - `mask`: Bitmask of error conditions to monitor (see `CanErrorMask`).
    ///
    /// # Returns
    /// A mutable reference to `self` on success, to allow call chaining.
    ///
    /// # Errors
    /// Returns a `CanError` if:
    /// - the underlying `setsockopt` call fails (e.g., due to insufficient
    ///   privileges, invalid arguments, or lack of kernel support);
    /// - the provided mask is empty/invalid for this platform;
    /// - the socket handle is not a valid CAN RAW socket;
    /// - the size of the option value does not match what the kernel expects.
    pub fn set_monitoring(&mut self, mask: &CanErrorMask) -> Result<&mut Self, CanError> {
        let flag = mask.bits();
        let status = unsafe {
            cglue::setsockopt(
                self.sockfd,
                i32::try_from(cglue::can_RAW_x_SOL_CAN_RAW).unwrap_or(i32::MAX),
                i32::try_from(mask.bits()).unwrap_or(i32::MAX),
                (&raw const flag).cast::<std::ffi::c_void>(),
                cglue::socklen_t::try_from(std::mem::size_of::<u32>()).unwrap_or(u32::MAX),
            )
        };
        if status < 0 {
            return Err(CanError::new("can-setsock-err-fail", cglue::get_perror()));
        }
        Ok(self)
    }

    pub fn get_any_frame(&self) -> CanAnyFrame {
        #[allow(invalid_value)]
        let buffer = [0u8; cglue::can_MTU_x_FD_MTU as usize];
        let count = unsafe {
            cglue::read(
                self.sockfd,
                (&raw const buffer).cast::<std::ffi::c_void>().cast_mut(),
                cglue::can_MTU_x_FD_MTU as usize,
            )
        };

        let sz_std = isize::try_from(std::mem::size_of::<CanFrameRaw>()).unwrap_or(isize::MAX);
        let sz_fd = isize::try_from(std::mem::size_of::<CanFdFrameRaw>()).unwrap_or(isize::MAX);

        #[allow(clippy::cast_ptr_alignment)]
        if count == sz_std {
            CanAnyFrame::from((&raw const buffer).cast::<CanFrameRaw>())
        } else if count == sz_fd {
            CanAnyFrame::from((&raw const buffer).cast::<CanFdFrameRaw>())
        } else {
            CanAnyFrame::Err(CanError::new("can-invalid-frame", cglue::get_perror()))
        }
    }

    pub fn get_raw_frame(&self, buffer: &mut [u8]) -> CanRecvInfo {
        let mut info: CanRecvInfo = unsafe { mem::zeroed::<CanRecvInfo>() };
        let _size = u32::try_from(buffer.len()).unwrap_or(u32::MAX);

        let mut control: cglue::can_stamp_msg = unsafe { std::mem::zeroed() };
        let mut canaddr: cglue::sockaddr_can = unsafe { std::mem::zeroed() };
        let mut msg_hdr: cglue::msghdr = unsafe { std::mem::zeroed() };

        msg_hdr.msg_flags = 0;
        let mut iov = cglue::iovec { iov_base: buffer.as_mut_ptr().cast(), iov_len: buffer.len() };

        msg_hdr.msg_iov = core::ptr::addr_of_mut!(iov);
        msg_hdr.msg_iovlen = 1;

        msg_hdr.msg_namelen =
            cglue::socklen_t::try_from(std::mem::size_of::<cglue::sockaddr_can>())
                .unwrap_or(u32::MAX);
        msg_hdr.msg_name = core::ptr::addr_of_mut!(canaddr).cast::<std::ffi::c_void>();
        msg_hdr.msg_control = core::ptr::addr_of_mut!(control).cast::<std::ffi::c_void>();
        msg_hdr.msg_controllen = std::mem::size_of::<cglue::can_stamp_msg>();

        info.count = unsafe {
            cglue::recvmsg(self.sockfd, (&raw const msg_hdr).cast::<cglue::msghdr>().cast_mut(), 0)
        };

        if msg_hdr.msg_namelen >= 8 {
            let _mutcanaddr = unsafe { &*msg_hdr.msg_name.cast::<cglue::sockaddr_can>() };
            info.iface = canaddr.can_ifindex;
        }

        if info.count < 0 {
            info.proto = CanProtoInfo::Error(CanError::new("can_read_frame", cglue::get_perror()));
            return info;
        }

        // ref: https://github.com/torvalds/linux/blob/master/tools/testing/selftests/net/timestamping.c
        //let mut cmsg = unsafe { cglue::CMSG_FIRSTHDR(&raw const msg_hdr) };
        let mut safe_msg = cglue::CMSG_FIRSTHDR(&raw const msg_hdr);

        while !safe_msg.is_null() {
            let c_msg = unsafe { &*safe_msg };

            // Constants converties en i32 une fois pour toutes (évite cast-sign-loss dans les comparaisons)
            let sol_socket: i32 = i32::try_from(cglue::can_SOCK_x_SOL_SOCKET).unwrap_or(i32::MAX);
            let so_timestamping: i32 =
                i32::try_from(cglue::can_RAW_x_SO_TIMESTAMPING).unwrap_or(i32::MAX);
            let so_timestamp_new: i32 =
                i32::try_from(cglue::can_RAW_x_SO_TIMESTAMP_NEW).unwrap_or(i32::MAX);
            let so_timestampns: i32 =
                i32::try_from(cglue::can_RAW_x_SO_TIMESTAMPNS).unwrap_or(i32::MAX);
            let so_timestamp: i32 =
                i32::try_from(cglue::can_RAW_x_SO_TIMESTAMP).unwrap_or(i32::MAX);

            let sol_can_j1939: i32 =
                i32::try_from(cglue::can_J1939_x_SOL_CAN_J1939).unwrap_or(i32::MAX);
            let scm_dest_addr: i32 =
                i32::try_from(cglue::can_J1939_x_SCM_DEST_ADDR).unwrap_or(i32::MAX);
            let scm_dest_name: i32 =
                i32::try_from(cglue::can_J1939_x_SCM_DEST_NAME).unwrap_or(i32::MAX);

            if c_msg.cmsg_level == sol_socket {
                let ctype = c_msg.cmsg_type;

                // Trois bras identiques fusionnés avec ||
                if ctype == so_timestamping || ctype == so_timestamp_new || ctype == so_timestampns
                {
                    // lire timespec sans exigence d’alignement strict
                    let ts = unsafe {
                        core::ptr::read_unaligned(cglue::CMSG_DATA(c_msg).cast::<cglue::timespec>())
                    };
                    let sec = u64::try_from(ts.tv_sec).unwrap_or(0);
                    let nsec = u64::try_from(ts.tv_nsec).unwrap_or(0);
                    info.stamp = sec.saturating_mul(1_000_000).saturating_add(nsec);
                    break;
                } else if ctype == so_timestamp {
                    // lire timeval sans exigence d’alignement strict
                    let tv = unsafe {
                        core::ptr::read_unaligned(cglue::CMSG_DATA(c_msg).cast::<cglue::timeval>())
                    };
                    let sec = u64::try_from(tv.tv_sec).unwrap_or(0);
                    let usec = u64::try_from(tv.tv_usec).unwrap_or(0);
                    info.stamp = sec.saturating_mul(1_000_000).saturating_add(usec);
                    break;
                }
            } else if c_msg.cmsg_level == sol_can_j1939 {
                info.iface = canaddr.can_ifindex;

                let j1939_src = unsafe {
                    CanJ1939Header {
                        name: canaddr.can_addr.j1939.name,
                        addr: canaddr.can_addr.j1939.addr,
                    }
                };
                let mut j1939_dst = CanJ1939Header { name: 0, addr: 0 };

                if c_msg.cmsg_type == scm_dest_addr {
                    let addr =
                        unsafe { core::ptr::read_unaligned(cglue::CMSG_DATA(c_msg).cast::<u8>()) };
                    j1939_dst.addr = addr;
                } else if c_msg.cmsg_type == scm_dest_name {
                    let name =
                        unsafe { core::ptr::read_unaligned(cglue::CMSG_DATA(c_msg).cast::<u64>()) };
                    j1939_dst.name = name;
                }

                info.proto = CanProtoInfo::J1939(CanJ1939Info {
                    src: j1939_src,
                    dst: j1939_dst,
                    pgn: unsafe { canaddr.can_addr.j1939.pgn },
                });
            }

            safe_msg = cglue::CMSG_NXTHDR(&raw const msg_hdr, core::ptr::from_ref(c_msg));
        }
        info
    }

    pub fn get_can_frame(&self) -> SockCanMsg {
        let mut buffer: [u8; cglue::can_MTU_x_FD_MTU as usize] =
            [0u8; cglue::can_MTU_x_FD_MTU as usize];
        let info = self.get_raw_frame(&mut buffer);

        let std_sz = isize::try_from(core::mem::size_of::<CanFrameRaw>()).unwrap_or(isize::MAX);
        let fd_sz = isize::try_from(core::mem::size_of::<CanFdFrameRaw>()).unwrap_or(isize::MAX);
        let can_any_frame = if info.count == std_sz {
            let mut tmp = core::mem::MaybeUninit::<CanFrameRaw>::uninit();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    buffer.as_ptr(),
                    tmp.as_mut_ptr().cast::<u8>(),
                    core::mem::size_of::<CanFrameRaw>(),
                );
                CanAnyFrame::RawStd(tmp.assume_init())
            }
        } else if info.count == fd_sz {
            let mut tmp = core::mem::MaybeUninit::<CanFdFrameRaw>::uninit();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    buffer.as_ptr(),
                    tmp.as_mut_ptr().cast::<u8>(),
                    core::mem::size_of::<CanFdFrameRaw>(),
                );
                CanAnyFrame::RawFd(tmp.assume_init())
            }
        } else {
            CanAnyFrame::Err(CanError::new("can-invalid-frame", cglue::get_perror()))
        };

        SockCanMsg { frame: can_any_frame, iface: info.iface, stamp: info.stamp }
    }
    /// Low-level send for Classical CAN
    pub fn send_std(&self, id: u32, data: &[u8]) -> Result<(), CanError> {
        let frame = build_std_frame(id, data)?;
        let n = unsafe {
            cglue::write(
                self.sockfd,
                (&raw const frame).cast::<std::ffi::c_void>(),
                core::mem::size_of::<cglue::can_frame>(),
            )
        };
        if n < 0 {
            return Err(CanError::new("can-send-std", cglue::get_perror()));
        }
        Ok(())
    }

    /// Low-level send for CAN FD
    pub fn send_fd(&self, id: u32, data: &[u8], flags: u8) -> Result<(), CanError> {
        let frame = build_fd_frame(id, data, flags)?;
        let n = unsafe {
            cglue::write(
                self.sockfd,
                (&raw const frame).cast::<std::ffi::c_void>(),
                core::mem::size_of::<cglue::canfd_frame>(),
            )
        };
        if n < 0 {
            return Err(CanError::new("can-send-fd", cglue::get_perror()));
        }
        Ok(())
    }

    /// Generic writer that accepts `CanAnyFrame` (what your reader returns).
    pub fn write_frame(&self, frame: &CanAnyFrame) -> Result<(), CanError> {
        match frame {
            CanAnyFrame::RawStd(f) => {
                let n = unsafe {
                    cglue::write(self.sockfd, f.as_ptr(), core::mem::size_of::<cglue::can_frame>())
                };
                if n < 0 {
                    return Err(CanError::new("can-send-std", cglue::get_perror()));
                }
                Ok(())
            },
            CanAnyFrame::RawFd(f) => {
                let n = unsafe {
                    cglue::write(
                        self.sockfd,
                        f.as_ptr(),
                        core::mem::size_of::<cglue::canfd_frame>(),
                    )
                };
                if n < 0 {
                    return Err(CanError::new("can-send-fd", cglue::get_perror()));
                }
                Ok(())
            },
            CanAnyFrame::Err(e) => Err(CanError::new("can-send-invalid", format!("Err: {e}"))),
            CanAnyFrame::None(id) => Err(CanError::new(
                "can-send-invalid",
                format!("None/timeout frame cannot be sent (id={id:08X})"),
            )),
        }
    }
}

impl SockCanFilter {
    #[must_use]
    pub fn new(size: usize) -> Self {
        if size > 0 {
            SockCanFilter { count: 0, masks: Vec::with_capacity(size) }
        } else {
            SockCanFilter { count: 0, masks: Vec::new() }
        }
    }

    /// Each filter contains an internal id and mask. Packets are considered to be matched
    /// by a filter if `received_id & mask == filter_id & mask` holds true.
    pub fn add_whitelist(&mut self, can_id: u32, can_mask: &FilterMask) -> &mut Self {
        self.count += 1;
        self.masks.push(cglue::can_filter { can_id, can_mask: can_mask.bits() });
        self
    }

    pub fn add_blacklist(&mut self, can_id: u32, can_mask: &FilterMask) -> &mut Self {
        self.count += 1;
        self.masks.push(cglue::can_filter {
            can_id: can_id | cglue::can_FILTER_x_INV_FILTER,
            can_mask: can_mask.bits(),
        });
        self
    }
    /// Applies the configured RAW socket options and filters to `sock`.
    ///
    /// This configures the underlying CAN RAW socket using the parameters
    /// previously set on `self` (e.g., blocking mode, timeouts, timestamping,
    /// loopback, error monitoring, filters/whitelist/blacklist, etc.).
    ///
    /// # Parameters
    /// - `sock`: An already opened `SockCanHandle` targeting the desired CAN interface.
    ///
    /// # Returns
    /// `Ok(())` if all options were successfully applied to the socket.
    ///
    /// # Errors
    /// Returns a `CanError` if any of the following occur while applying options:
    /// - the socket handle is invalid or not a RAW CAN socket;
    /// - a `setsockopt`/`ioctl` call fails (e.g., insufficient privileges,
    ///   unsupported option on this kernel, or invalid argument values);
    /// - filter installation fails (e.g., empty/invalid filter set, size mismatch);
    /// - timestamp/timeout/loopback/error-mask configuration cannot be applied;
    /// - any system call returns an unexpected error code that cannot be mapped.
    pub fn apply(&mut self, sock: &SockCanHandle) -> Result<(), CanError> {
        match sock.mode {
            SockCanMod::RAW => {},
            _ => return Err(CanError::new("invalid-socketcan-mod", "not a RAW socket can")),
        }

        let filter_ptr = self.masks.as_ptr();
        let status = unsafe {
            cglue::setsockopt(
                sock.sockfd,
                i32::try_from(cglue::can_RAW_x_SOL_CAN_RAW).unwrap_or(i32::MAX),
                i32::try_from(cglue::can_RAW_x_FILTER).unwrap_or(i32::MAX),
                filter_ptr.cast::<::std::os::raw::c_void>(),
                cglue::socklen_t::try_from(core::mem::size_of::<cglue::can_filter>() * self.count)
                    .unwrap_or(u32::MAX),
            )
        };
        if status < 0 {
            return Err(CanError::new("fail-socketcan-bind", cglue::get_perror()));
        }
        Ok(())
    }
}
