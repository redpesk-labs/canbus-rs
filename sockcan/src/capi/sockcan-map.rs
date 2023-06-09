/* automatically generated by rust-bindgen 0.65.1 */


    // -----------------------------------------------------------------------
    //         <- private 'sockcan' Rust/C unsafe binding ->
    // -----------------------------------------------------------------------
    //   Do not exit this file it will be regenerated automatically by cargo.
    //   Check:
    //     - build.rs at project root for dynamically mapping
    //     - src/capi/sockcan-map.h for static values
    // -----------------------------------------------------------------------
    

#[repr(C)]
#[derive(Default)]
pub struct __IncompleteArrayField<T>(::std::marker::PhantomData<T>, [T; 0]);
impl<T> __IncompleteArrayField<T> {
    #[inline]
    pub const fn new() -> Self {
        __IncompleteArrayField(::std::marker::PhantomData, [])
    }
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self as *const _ as *const T
    }
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self as *mut _ as *mut T
    }
    #[inline]
    pub unsafe fn as_slice(&self, len: usize) -> &[T] {
        ::std::slice::from_raw_parts(self.as_ptr(), len)
    }
    #[inline]
    pub unsafe fn as_mut_slice(&mut self, len: usize) -> &mut [T] {
        ::std::slice::from_raw_parts_mut(self.as_mut_ptr(), len)
    }
}
impl<T> ::std::fmt::Debug for __IncompleteArrayField<T> {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        fmt.write_str("__IncompleteArrayField")
    }
}
pub type __time_t = ::std::os::raw::c_long;
pub type __suseconds_t = ::std::os::raw::c_long;
pub type __syscall_slong_t = ::std::os::raw::c_long;
pub type __caddr_t = *mut ::std::os::raw::c_char;
pub type __socklen_t = ::std::os::raw::c_uint;
pub type time_t = __time_t;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}
extern "C" {
    pub fn strerror_r(
        __errnum: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
        __buflen: usize,
    ) -> *mut ::std::os::raw::c_char;
}
pub type socklen_t = __socklen_t;
extern "C" {
    pub fn close(__fd: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn read(
        __fd: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_void,
        __nbytes: usize,
    ) -> isize;
}
extern "C" {
    pub fn write(
        __fd: ::std::os::raw::c_int,
        __buf: *const ::std::os::raw::c_void,
        __n: usize,
    ) -> isize;
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct iovec {
    pub iov_base: *mut ::std::os::raw::c_void,
    pub iov_len: usize,
}
extern "C" {
    pub fn fcntl(
        __fd: ::std::os::raw::c_int,
        __cmd: ::std::os::raw::c_int,
        ...
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn __errno_location() -> *mut ::std::os::raw::c_int;
}
pub type sa_family_t = ::std::os::raw::c_ushort;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    pub sa_data: [::std::os::raw::c_char; 14usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct msghdr {
    pub msg_name: *mut ::std::os::raw::c_void,
    pub msg_namelen: socklen_t,
    pub msg_iov: *mut iovec,
    pub msg_iovlen: usize,
    pub msg_control: *mut ::std::os::raw::c_void,
    pub msg_controllen: usize,
    pub msg_flags: ::std::os::raw::c_int,
}
#[repr(C)]
pub struct cmsghdr {
    pub cmsg_len: usize,
    pub cmsg_level: ::std::os::raw::c_int,
    pub cmsg_type: ::std::os::raw::c_int,
    pub __cmsg_data: __IncompleteArrayField<::std::os::raw::c_uchar>,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union __SOCKADDR_ARG {
    pub __sockaddr__: *mut sockaddr,
    pub __sockaddr_at__: *mut sockaddr_at,
    pub __sockaddr_ax25__: *mut sockaddr_ax25,
    pub __sockaddr_dl__: *mut sockaddr_dl,
    pub __sockaddr_eon__: *mut sockaddr_eon,
    pub __sockaddr_in__: *mut sockaddr_in,
    pub __sockaddr_in6__: *mut sockaddr_in6,
    pub __sockaddr_inarp__: *mut sockaddr_inarp,
    pub __sockaddr_ipx__: *mut sockaddr_ipx,
    pub __sockaddr_iso__: *mut sockaddr_iso,
    pub __sockaddr_ns__: *mut sockaddr_ns,
    pub __sockaddr_un__: *mut sockaddr_un,
    pub __sockaddr_x25__: *mut sockaddr_x25,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union __CONST_SOCKADDR_ARG {
    pub __sockaddr__: *const sockaddr,
    pub __sockaddr_at__: *const sockaddr_at,
    pub __sockaddr_ax25__: *const sockaddr_ax25,
    pub __sockaddr_dl__: *const sockaddr_dl,
    pub __sockaddr_eon__: *const sockaddr_eon,
    pub __sockaddr_in__: *const sockaddr_in,
    pub __sockaddr_in6__: *const sockaddr_in6,
    pub __sockaddr_inarp__: *const sockaddr_inarp,
    pub __sockaddr_ipx__: *const sockaddr_ipx,
    pub __sockaddr_iso__: *const sockaddr_iso,
    pub __sockaddr_ns__: *const sockaddr_ns,
    pub __sockaddr_un__: *const sockaddr_un,
    pub __sockaddr_x25__: *const sockaddr_x25,
}
extern "C" {
    pub fn socket(
        __domain: ::std::os::raw::c_int,
        __type: ::std::os::raw::c_int,
        __protocol: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn bind(
        __fd: ::std::os::raw::c_int,
        __addr: __CONST_SOCKADDR_ARG,
        __len: socklen_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn connect(
        __fd: ::std::os::raw::c_int,
        __addr: __CONST_SOCKADDR_ARG,
        __len: socklen_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn send(
        __fd: ::std::os::raw::c_int,
        __buf: *const ::std::os::raw::c_void,
        __n: usize,
        __flags: ::std::os::raw::c_int,
    ) -> isize;
}
extern "C" {
    pub fn recvfrom(
        __fd: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_void,
        __n: usize,
        __flags: ::std::os::raw::c_int,
        __addr: __SOCKADDR_ARG,
        __addr_len: *mut socklen_t,
    ) -> isize;
}
extern "C" {
    pub fn recvmsg(
        __fd: ::std::os::raw::c_int,
        __message: *mut msghdr,
        __flags: ::std::os::raw::c_int,
    ) -> isize;
}
extern "C" {
    pub fn setsockopt(
        __fd: ::std::os::raw::c_int,
        __level: ::std::os::raw::c_int,
        __optname: ::std::os::raw::c_int,
        __optval: *const ::std::os::raw::c_void,
        __optlen: socklen_t,
    ) -> ::std::os::raw::c_int;
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifmap {
    pub mem_start: ::std::os::raw::c_ulong,
    pub mem_end: ::std::os::raw::c_ulong,
    pub base_addr: ::std::os::raw::c_ushort,
    pub irq: ::std::os::raw::c_uchar,
    pub dma: ::std::os::raw::c_uchar,
    pub port: ::std::os::raw::c_uchar,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifreq {
    pub ifr_ifrn: ifreq__bindgen_ty_1,
    pub ifr_ifru: ifreq__bindgen_ty_2,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union ifreq__bindgen_ty_1 {
    pub ifrn_name: [::std::os::raw::c_char; 16usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union ifreq__bindgen_ty_2 {
    pub ifru_addr: sockaddr,
    pub ifru_dstaddr: sockaddr,
    pub ifru_broadaddr: sockaddr,
    pub ifru_netmask: sockaddr,
    pub ifru_hwaddr: sockaddr,
    pub ifru_flags: ::std::os::raw::c_short,
    pub ifru_ivalue: ::std::os::raw::c_int,
    pub ifru_mtu: ::std::os::raw::c_int,
    pub ifru_map: ifmap,
    pub ifru_slave: [::std::os::raw::c_char; 16usize],
    pub ifru_newname: [::std::os::raw::c_char; 16usize],
    pub ifru_data: __caddr_t,
}
extern "C" {
    pub fn ioctl(
        __fd: ::std::os::raw::c_int,
        __request: ::std::os::raw::c_ulong,
        ...
    ) -> ::std::os::raw::c_int;
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct tm {
    pub tm_sec: ::std::os::raw::c_int,
    pub tm_min: ::std::os::raw::c_int,
    pub tm_hour: ::std::os::raw::c_int,
    pub tm_mday: ::std::os::raw::c_int,
    pub tm_mon: ::std::os::raw::c_int,
    pub tm_year: ::std::os::raw::c_int,
    pub tm_wday: ::std::os::raw::c_int,
    pub tm_yday: ::std::os::raw::c_int,
    pub tm_isdst: ::std::os::raw::c_int,
    pub tm_gmtoff: ::std::os::raw::c_long,
    pub tm_zone: *const ::std::os::raw::c_char,
}
extern "C" {
    pub fn time(__timer: *mut time_t) -> time_t;
}
extern "C" {
    pub fn strftime(
        __s: *mut ::std::os::raw::c_char,
        __maxsize: usize,
        __format: *const ::std::os::raw::c_char,
        __tp: *const tm,
    ) -> usize;
}
extern "C" {
    pub fn localtime(__timer: *const time_t) -> *mut tm;
}
pub type __u8 = ::std::os::raw::c_uchar;
pub type __u16 = ::std::os::raw::c_ushort;
pub type __u32 = ::std::os::raw::c_uint;
pub type __u64 = ::std::os::raw::c_ulonglong;
pub type __kernel_sa_family_t = ::std::os::raw::c_ushort;
pub type canid_t = __u32;
pub type can_err_mask_t = __u32;
#[repr(C)]
#[repr(align(8))]
#[derive(Copy, Clone)]
pub struct can_frame {
    pub can_id: canid_t,
    pub __bindgen_anon_1: can_frame__bindgen_ty_1,
    pub __pad: __u8,
    pub __res0: __u8,
    pub len8_dlc: __u8,
    pub data: [__u8; 8usize],
}
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub union can_frame__bindgen_ty_1 {
    pub len: __u8,
    pub can_dlc: __u8,
}
#[repr(C)]
#[repr(align(8))]
#[derive(Copy, Clone)]
pub struct canfd_frame {
    pub can_id: canid_t,
    pub len: __u8,
    pub flags: __u8,
    pub __res0: __u8,
    pub __res1: __u8,
    pub data: [__u8; 64usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_can {
    pub can_family: __kernel_sa_family_t,
    pub can_ifindex: ::std::os::raw::c_int,
    pub can_addr: sockaddr_can__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union sockaddr_can__bindgen_ty_1 {
    pub tp: sockaddr_can__bindgen_ty_1__bindgen_ty_1,
    pub j1939: sockaddr_can__bindgen_ty_1__bindgen_ty_2,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_can__bindgen_ty_1__bindgen_ty_1 {
    pub rx_id: canid_t,
    pub tx_id: canid_t,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_can__bindgen_ty_1__bindgen_ty_2 {
    pub name: __u64,
    pub pgn: __u32,
    pub addr: __u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_filter {
    pub can_id: canid_t,
    pub can_mask: canid_t,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct bcm_timeval {
    pub tv_sec: ::std::os::raw::c_long,
    pub tv_usec: ::std::os::raw::c_long,
}
#[repr(C)]
pub struct bcm_msg_head {
    pub opcode: __u32,
    pub flags: __u32,
    pub count: __u32,
    pub ival1: bcm_timeval,
    pub ival2: bcm_timeval,
    pub can_id: canid_t,
    pub nframes: __u32,
    pub frames: __IncompleteArrayField<can_frame>,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_isotp_options {
    pub flags: __u32,
    pub frame_txtime: __u32,
    pub ext_address: __u8,
    pub txpad_content: __u8,
    pub rxpad_content: __u8,
    pub rx_ext_address: __u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_isotp_fc_options {
    pub bs: __u8,
    pub stmin: __u8,
    pub wftmax: __u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_isotp_ll_options {
    pub mtu: __u8,
    pub tx_dl: __u8,
    pub tx_flags: __u8,
}
pub type pgn_t = __u32;
pub type name_t = __u64;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct j1939_filter {
    pub name: name_t,
    pub name_mask: name_t,
    pub pgn: pgn_t,
    pub pgn_mask: pgn_t,
    pub addr: __u8,
    pub addr_mask: __u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_bittiming {
    pub bitrate: __u32,
    pub sample_point: __u32,
    pub tq: __u32,
    pub prop_seg: __u32,
    pub phase_seg1: __u32,
    pub phase_seg2: __u32,
    pub sjw: __u32,
    pub brp: __u32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_bittiming_const {
    pub name: [::std::os::raw::c_char; 16usize],
    pub tseg1_min: __u32,
    pub tseg1_max: __u32,
    pub tseg2_min: __u32,
    pub tseg2_max: __u32,
    pub sjw_max: __u32,
    pub brp_min: __u32,
    pub brp_max: __u32,
    pub brp_inc: __u32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_clock {
    pub freq: __u32,
}
pub const can_state_CAN_STATE_ERROR_ACTIVE: can_state = 0;
pub const can_state_CAN_STATE_ERROR_WARNING: can_state = 1;
pub const can_state_CAN_STATE_ERROR_PASSIVE: can_state = 2;
pub const can_state_CAN_STATE_BUS_OFF: can_state = 3;
pub const can_state_CAN_STATE_STOPPED: can_state = 4;
pub const can_state_CAN_STATE_SLEEPING: can_state = 5;
pub const can_state_CAN_STATE_MAX: can_state = 6;
pub type can_state = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_berr_counter {
    pub txerr: __u16,
    pub rxerr: __u16,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_ctrlmode {
    pub mask: __u32,
    pub flags: __u32,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct can_device_stats {
    pub bus_error: __u32,
    pub error_warning: __u32,
    pub error_passive: __u32,
    pub bus_off: __u32,
    pub arbitration_lost: __u32,
    pub restarts: __u32,
}
pub type can_cmsghdr = cmsghdr;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct cmsg {
    _unused: [u8; 0],
}
pub type can_cmsg = cmsg;
pub type can_timeval = timeval;
pub type can_timespec = timespec;
#[repr(C)]
pub struct can_bcm_msg {
    pub head: bcm_msg_head,
    pub frames: [can_frame; 128usize],
}
#[repr(C)]
pub struct canfd_bcm_msg {
    pub head: bcm_msg_head,
    pub fdframes: [canfd_frame; 128usize],
}
#[repr(C)]
pub struct can_bcm_one_msg {
    pub head: bcm_msg_head,
    pub frame: can_frame,
}
#[repr(C)]
pub struct canfd_bcm_one_msg {
    pub head: bcm_msg_head,
    pub frame: canfd_frame,
}
#[repr(C)]
pub struct can_stamp_msg {
    pub head: cmsghdr,
    pub control: [::std::os::raw::c_char; 32usize],
}
pub const can_CMSG_CMSG_DATA: can_CMSG = 0;
pub const can_CMSG_CMSG_FIRSTHDR: can_CMSG = 1;
pub const can_CMSG_CMSG_NXTHDR: can_CMSG = 2;
pub type can_CMSG = ::std::os::raw::c_uint;
pub const can_SOCK_x_PF_CAN: can_SOCK = 29;
pub const can_SOCK_x_AF_CAN: can_SOCK = 29;
pub const can_SOCK_x_RAW: can_SOCK = 3;
pub const can_SOCK_x_DGRAM: can_SOCK = 2;
pub const can_SOCK_x_IFACE_LEN: can_SOCK = 16;
pub const can_SOCK_x_SIOCGIFINDEX: can_SOCK = 35123;
pub const can_SOCK_x_SIOCGIFNAME: can_SOCK = 35088;
pub const can_SOCK_x_SIOCGSTAMP: can_SOCK = 35078;
pub const can_SOCK_x_SO_BROADCAST: can_SOCK = 6;
pub const can_SOCK_x_SFF_ID_BITS: can_SOCK = 11;
pub const can_SOCK_x_EFF_ID_BITS: can_SOCK = 29;
pub const can_SOCK_x_MAX_DLC: can_SOCK = 8;
pub const can_SOCK_x_MAX_RAW_DLC: can_SOCK = 15;
pub const can_SOCK_x_MAX_DLEN: can_SOCK = 8;
pub const can_SOCK_x_FD_MAX_DLC: can_SOCK = 15;
pub const can_SOCK_x_FD_MAX_DLEN: can_SOCK = 64;
pub const can_SOCK_x_F_GETFL: can_SOCK = 3;
pub const can_SOCK_x_F_SETFL: can_SOCK = 4;
pub const can_SOCK_x_NONBLOCK: can_SOCK = 2048;
pub const can_SOCK_x_SOL_SOCKET: can_SOCK = 1;
pub const can_SOCK_x_SO_SNDTIMEO: can_SOCK = 21;
pub const can_SOCK_x_SO_RCVTIMEO: can_SOCK = 20;
pub const can_SOCK_x_MSG_EOR: can_SOCK = 128;
pub const can_SOCK_x_MAX_BCM_CAN_FRAMES: can_SOCK = 128;
pub const can_SOCK_x_MAX_ISOTP_FRAMES: can_SOCK = 4096;
pub const can_SOCK_x_SIOCGIFMTU: can_SOCK = 35105;
pub const can_SOCK_x_TP16: can_SOCK = 3;
pub const can_SOCK_x_TP20: can_SOCK = 4;
pub const can_SOCK_x_MCNET: can_SOCK = 5;
pub const can_SOCK_x_CANRAW: can_SOCK = 1;
pub const can_SOCK_x_BCM: can_SOCK = 2;
pub const can_SOCK_x_ISOTP: can_SOCK = 6;
pub const can_SOCK_x_J1939: can_SOCK = 7;
pub const can_SOCK_x_NPROTO: can_SOCK = 8;
pub type can_SOCK = ::std::os::raw::c_uint;
pub const can_MASK_x_SFF_MASK: can_MASK = 2047;
pub const can_MASK_x_EFF_MASK: can_MASK = 536870911;
pub const can_MASK_x_ERR_MASK: can_MASK = 536870911;
pub type can_MASK = ::std::os::raw::c_uint;
pub const can_MTU_x_MTU: can_MTU = 16;
pub const can_MTU_x_FD_MTU: can_MTU = 72;
pub type can_MTU = ::std::os::raw::c_uint;
pub const can_FILTER_x_INV_FILTER: can_FILTER = 536870912;
pub const can_FILTER_x_RAW_FILTER_MAX: can_FILTER = 512;
pub type can_FILTER = ::std::os::raw::c_uint;
pub const can_FLAGS_x_FD_BRS: can_FLAGS = 1;
pub const can_FLAGS_x_FD_ESI: can_FLAGS = 2;
pub const can_FLAGS_x_FD_FDF: can_FLAGS = 4;
pub const can_FLAGS_x_EFF_FLAG: can_FLAGS = 2147483648;
pub const can_FLAGS_x_RTR_FLAG: can_FLAGS = 1073741824;
pub const can_FLAGS_x_ERR_FLAG: can_FLAGS = 536870912;
pub type can_FLAGS = ::std::os::raw::c_uint;
pub const can_BCM_FLAG_x_SETTIMER: can_BCM_FLAG = 1;
pub const can_BCM_FLAG_x_STARTTIMER: can_BCM_FLAG = 2;
pub const can_BCM_FLAG_x_TX_COUNTEVT: can_BCM_FLAG = 4;
pub const can_BCM_FLAG_x_TX_ANNOUNCE: can_BCM_FLAG = 8;
pub const can_BCM_FLAG_x_TX_CP_CAN_ID: can_BCM_FLAG = 16;
pub const can_BCM_FLAG_x_RX_FILTER_ID: can_BCM_FLAG = 32;
pub const can_BCM_FLAG_x_RX_CHECK_DLC: can_BCM_FLAG = 64;
pub const can_BCM_FLAG_x_RX_NO_AUTOTIMER: can_BCM_FLAG = 128;
pub const can_BCM_FLAG_x_RX_ANNOUNCE_RESUME: can_BCM_FLAG = 256;
pub const can_BCM_FLAG_x_TX_RESET_MULTI_IDX: can_BCM_FLAG = 512;
pub const can_BCM_FLAG_x_RX_RTR_FRAME: can_BCM_FLAG = 1024;
pub const can_BCM_FLAG_x_FD_FRAME: can_BCM_FLAG = 2048;
pub type can_BCM_FLAG = ::std::os::raw::c_uint;
pub const can_BCM_OPE_x_TX_SETUP: can_BCM_OPE = 1;
pub const can_BCM_OPE_x_TX_DELETE: can_BCM_OPE = 2;
pub const can_BCM_OPE_x_TX_READ: can_BCM_OPE = 3;
pub const can_BCM_OPE_x_TX_SEND: can_BCM_OPE = 4;
pub const can_BCM_OPE_x_TX_STATUS: can_BCM_OPE = 8;
pub const can_BCM_OPE_x_TX_EXPIRED: can_BCM_OPE = 9;
pub const can_BCM_OPE_x_RX_SETUP: can_BCM_OPE = 5;
pub const can_BCM_OPE_x_RX_DELETE: can_BCM_OPE = 6;
pub const can_BCM_OPE_x_RX_READ: can_BCM_OPE = 7;
pub const can_BCM_OPE_x_RX_STATUS: can_BCM_OPE = 10;
pub const can_BCM_OPE_x_RX_TIMEOUT: can_BCM_OPE = 11;
pub const can_BCM_OPE_x_RX_CHANGED: can_BCM_OPE = 12;
pub type can_BCM_OPE = ::std::os::raw::c_uint;
pub const can_ERROR_x_DLC: can_ERROR = 8;
pub const can_ERROR_x_TX_TIMEOUT: can_ERROR = 1;
pub const can_ERROR_x_LOSTARB: can_ERROR = 2;
pub const can_ERROR_x_CRTL: can_ERROR = 4;
pub const can_ERROR_x_PROT: can_ERROR = 8;
pub const can_ERROR_x_TRX: can_ERROR = 16;
pub const can_ERROR_x_ACK: can_ERROR = 32;
pub const can_ERROR_x_BUSOFF: can_ERROR = 64;
pub const can_ERROR_x_BUSERROR: can_ERROR = 128;
pub const can_ERROR_x_RESTARTED: can_ERROR = 256;
pub const can_ERROR_x_LOSTARB_UNSPEC: can_ERROR = 0;
pub type can_ERROR = ::std::os::raw::c_uint;
pub const can_CTRL_x_CRTL_UNSPEC: can_CTRL = 0;
pub const can_CTRL_x_CRTL_RX_OVERFLOW: can_CTRL = 1;
pub const can_CTRL_x_CRTL_TX_OVERFLOW: can_CTRL = 2;
pub const can_CTRL_x_CRTL_RX_WARNING: can_CTRL = 4;
pub const can_CTRL_x_CRTL_TX_WARNING: can_CTRL = 8;
pub const can_CTRL_x_CRTL_RX_PASSIVE: can_CTRL = 16;
pub const can_CTRL_x_CRTL_TX_PASSIVE: can_CTRL = 32;
pub const can_CTRL_x_CRTL_ACTIVE: can_CTRL = 64;
pub const can_CTRL_x_PROT_UNSPEC: can_CTRL = 0;
pub const can_CTRL_x_PROT_BIT: can_CTRL = 1;
pub const can_CTRL_x_PROT_FORM: can_CTRL = 2;
pub const can_CTRL_x_PROT_STUFF: can_CTRL = 4;
pub const can_CTRL_x_PROT_BIT0: can_CTRL = 8;
pub const can_CTRL_x_PROT_BIT1: can_CTRL = 16;
pub const can_CTRL_x_PROT_OVERLOAD: can_CTRL = 32;
pub const can_CTRL_x_PROT_ACTIVE: can_CTRL = 64;
pub const can_CTRL_x_PROT_TX: can_CTRL = 128;
pub const can_CTRL_x_PROT_LOC_UNSPEC: can_CTRL = 0;
pub const can_CTRL_x_PROT_LOC_SOF: can_CTRL = 3;
pub const can_CTRL_x_PROT_LOC_ID28_21: can_CTRL = 2;
pub const can_CTRL_x_PROT_LOC_ID20_18: can_CTRL = 6;
pub const can_CTRL_x_PROT_LOC_SRTR: can_CTRL = 4;
pub const can_CTRL_x_PROT_LOC_IDE: can_CTRL = 5;
pub const can_CTRL_x_PROT_LOC_ID17_13: can_CTRL = 7;
pub const can_CTRL_x_PROT_LOC_ID12_05: can_CTRL = 15;
pub const can_CTRL_x_PROT_LOC_ID04_00: can_CTRL = 14;
pub const can_CTRL_x_PROT_LOC_RTR: can_CTRL = 12;
pub const can_CTRL_x_PROT_LOC_RES1: can_CTRL = 13;
pub const can_CTRL_x_PROT_LOC_RES0: can_CTRL = 9;
pub const can_CTRL_x_PROT_LOC_DLC: can_CTRL = 11;
pub const can_CTRL_x_PROT_LOC_DATA: can_CTRL = 10;
pub const can_CTRL_x_PROT_LOC_CRC_SEQ: can_CTRL = 8;
pub const can_CTRL_x_PROT_LOC_CRC_DEL: can_CTRL = 24;
pub const can_CTRL_x_PROT_LOC_ACK: can_CTRL = 25;
pub const can_CTRL_x_PROT_LOC_ACK_DEL: can_CTRL = 27;
pub const can_CTRL_x_PROT_LOC_EOF: can_CTRL = 26;
pub const can_CTRL_x_PROT_LOC_INTERM: can_CTRL = 18;
pub const can_CTRL_x_TRX_UNSPEC: can_CTRL = 0;
pub const can_CTRL_x_TRX_CANH_NO_WIRE: can_CTRL = 4;
pub const can_CTRL_x_TRX_CANH_SHORT_TO_BAT: can_CTRL = 5;
pub const can_CTRL_x_TRX_CANH_SHORT_TO_VCC: can_CTRL = 6;
pub const can_CTRL_x_TRX_CANH_SHORT_TO_GND: can_CTRL = 7;
pub const can_CTRL_x_TRX_CANL_NO_WIRE: can_CTRL = 64;
pub const can_CTRL_x_TRX_CANL_SHORT_TO_BAT: can_CTRL = 80;
pub const can_CTRL_x_TRX_CANL_SHORT_TO_VCC: can_CTRL = 96;
pub const can_CTRL_x_TRX_CANL_SHORT_TO_GND: can_CTRL = 112;
pub const can_CTRL_x_TRX_CANL_SHORT_TO_CANH: can_CTRL = 128;
pub type can_CTRL = ::std::os::raw::c_uint;
pub const can_CGW_x_TYPE_MAX: can_CGW = 1;
pub const can_CGW_x_MAX: can_CGW = 18;
pub const can_CGW_x_FLAGS_CAN_ECHO: can_CGW = 1;
pub const can_CGW_x_FLAGS_CAN_SRC_TSTAMP: can_CGW = 2;
pub const can_CGW_x_FLAGS_CAN_IIF_TX_OK: can_CGW = 4;
pub const can_CGW_x_FLAGS_CAN_FD: can_CGW = 8;
pub const can_CGW_x_MOD_FUNCS: can_CGW = 4;
pub const can_CGW_x_MOD_ID: can_CGW = 1;
pub const can_CGW_x_MOD_DLC: can_CGW = 2;
pub const can_CGW_x_MOD_LEN: can_CGW = 2;
pub const can_CGW_x_MOD_DATA: can_CGW = 4;
pub const can_CGW_x_MOD_FLAGS: can_CGW = 8;
pub const can_CGW_x_FRAME_MODS: can_CGW = 4;
pub const can_CGW_can_MAX_MODFUNCTIONS: can_CGW = 16;
pub const can_CGW_x_MODATTR_LEN: can_CGW = 17;
pub const can_CGW_x_FDMODATTR_LEN: can_CGW = 73;
pub const can_CGW_x_CS_XOR_LEN: can_CGW = 4;
pub const can_CGW_x_CS_CRC8_LEN: can_CGW = 282;
pub const can_CGW_x_CRC8PRF_MAX: can_CGW = 3;
pub type can_CGW = ::std::os::raw::c_uint;
pub const can_ISOTP_x_SOL_ISOTP: can_ISOTP = 106;
pub const can_ISOTP_x_OPTS: can_ISOTP = 1;
pub const can_ISOTP_x_RECV_FC: can_ISOTP = 2;
pub const can_ISOTP_x_TX_STMIN: can_ISOTP = 3;
pub const can_ISOTP_x_RX_STMIN: can_ISOTP = 4;
pub const can_ISOTP_x_LL_OPTS: can_ISOTP = 5;
pub const can_ISOTP_x_LISTEN_MODE: can_ISOTP = 1;
pub const can_ISOTP_x_EXTEND_ADDR: can_ISOTP = 2;
pub const can_ISOTP_x_TX_PADDING: can_ISOTP = 4;
pub const can_ISOTP_x_RX_PADDING: can_ISOTP = 8;
pub const can_ISOTP_x_CHK_PAD_LEN: can_ISOTP = 16;
pub const can_ISOTP_x_CHK_PAD_DATA: can_ISOTP = 32;
pub const can_ISOTP_x_HALF_DUPLEX: can_ISOTP = 64;
pub const can_ISOTP_x_FORCE_TXSTMIN: can_ISOTP = 128;
pub const can_ISOTP_x_FORCE_RXSTMIN: can_ISOTP = 256;
pub const can_ISOTP_x_RX_EXT_ADDR: can_ISOTP = 512;
pub const can_ISOTP_x_WAIT_TX_DONE: can_ISOTP = 1024;
pub const can_ISOTP_x_SF_BROADCAST: can_ISOTP = 2048;
pub const can_ISOTP_x_DEFAULT_FLAGS: can_ISOTP = 0;
pub const can_ISOTP_x_DEFAULT_EXT_ADDRESS: can_ISOTP = 0;
pub const can_ISOTP_x_DEFAULT_PAD_CONTENT: can_ISOTP = 204;
pub const can_ISOTP_x_DEFAULT_FRAME_TXTIME: can_ISOTP = 50000;
pub const can_ISOTP_x_DEFAULT_RECV_BS: can_ISOTP = 0;
pub const can_ISOTP_x_DEFAULT_RECV_STMIN: can_ISOTP = 0;
pub const can_ISOTP_x_DEFAULT_RECV_WFTMAX: can_ISOTP = 0;
pub const can_ISOTP_x_DEFAULT_LL_MTU: can_ISOTP = 16;
pub const can_ISOTP_x_DEFAULT_LL_TX_DL: can_ISOTP = 8;
pub const can_ISOTP_x_DEFAULT_LL_TX_FLAGS: can_ISOTP = 0;
pub type can_ISOTP = ::std::os::raw::c_uint;
pub const can_J1939_x_MAX_UNICAST_ADDR: can_J1939 = 253;
pub const can_J1939_x_IDLE_ADDR: can_J1939 = 254;
pub const can_J1939_x_NO_ADDR: can_J1939 = 255;
pub const can_J1939_x_NO_NAME: can_J1939 = 0;
pub const can_J1939_x_PGN_REQUEST: can_J1939 = 59904;
pub const can_J1939_x_PGN_ADDRESS_CLAIMED: can_J1939 = 60928;
pub const can_J1939_x_PGN_ADDRESS_COMMANDED: can_J1939 = 65240;
pub const can_J1939_x_PGN_PDU1_MAX: can_J1939 = 261888;
pub const can_J1939_x_PGN_MAX: can_J1939 = 262143;
pub const can_J1939_x_NO_PGN: can_J1939 = 262144;
pub const can_J1939_x_SOL_CAN_J1939: can_J1939 = 107;
pub const can_J1939_x_FILTER_MAX: can_J1939 = 512;
pub const can_J1939_x_SO_FILTER: can_J1939 = 1;
pub const can_J1939_x_SO_PROMISC: can_J1939 = 2;
pub const can_J1939_x_SO_SEND_PRIO: can_J1939 = 3;
pub const can_J1939_x_SO_ERRQUEUE: can_J1939 = 4;
pub const can_J1939_x_MAX_TP_PACKET_SIZE: can_J1939 = 1785;
pub const can_J1939_x_MAX_ETP_PACKET_SIZE: can_J1939 = 117440505;
pub const can_J1939_x_SCM_DEST_ADDR: can_J1939 = 1;
pub const can_J1939_x_SCM_DEST_NAME: can_J1939 = 2;
pub const can_J1939_x_SCM_PRIO: can_J1939 = 3;
pub type can_J1939 = ::std::os::raw::c_uint;
pub const can_NETLINK_x_CTRLMODE_LOOPBACK: can_NETLINK = 1;
pub const can_NETLINK_x_CTRLMODE_LISTENONLY: can_NETLINK = 2;
pub const can_NETLINK_x_CTRLMODE_3_SAMPLES: can_NETLINK = 4;
pub const can_NETLINK_x_CTRLMODE_ONE_SHOT: can_NETLINK = 8;
pub const can_NETLINK_x_CTRLMODE_BERR_REPORTING: can_NETLINK = 16;
pub const can_NETLINK_x_CTRLMODE_FD: can_NETLINK = 32;
pub const can_NETLINK_x_CTRLMODE_PRESUME_ACK: can_NETLINK = 64;
pub const can_NETLINK_x_CTRLMODE_FD_NON_ISO: can_NETLINK = 128;
pub const can_NETLINK_x_CTRLMODE_CC_LEN8_DLC: can_NETLINK = 256;
pub const can_NETLINK_x_IFLA_CAN_MAX: can_NETLINK = 17;
pub const can_NETLINK_x_TERMINATION_DISABLED: can_NETLINK = 0;
pub type can_NETLINK = ::std::os::raw::c_uint;
pub const can_RAW_x_SOL_CAN_RAW: can_RAW = 101;
pub const can_RAW_x_LOOPBACK: can_RAW = 3;
pub const can_RAW_x_RECV_OWN_MSGS: can_RAW = 4;
pub const can_RAW_x_FILTER: can_RAW = 1;
pub const can_RAW_x_SO_TIMESTAMP: can_RAW = 29;
pub const can_RAW_x_SO_TIMESTAMPNS: can_RAW = 35;
pub const can_RAW_x_SO_TIMESTAMPING: can_RAW = 37;
pub const can_RAW_x_SO_TIMESTAMP_NEW: can_RAW = 63;
pub const can_RAW_x_SOF_TIMESTAMPING_RX_HARDWARE: can_RAW = 4;
pub const can_RAW_x_SOF_TIMESTAMPING_RX_SOFTWARE: can_RAW = 8;
pub type can_RAW = ::std::os::raw::c_uint;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_at {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_ax25 {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_dl {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_eon {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_in {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_in6 {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_inarp {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_ipx {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_iso {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_ns {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_un {
    pub _address: u8,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct sockaddr_x25 {
    pub _address: u8,
}
