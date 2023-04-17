/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk interface code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 *
*/
extern crate bindgen;
extern crate cc;

fn main() {
    // add here any special search path specific to your configuration
    println!("cargo:rustc-link-search=/usr/local/lib64");

    // invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/capi/sockcan-map.h");

    let header = "
    // -----------------------------------------------------------------------
    //         <- private 'sockcan' Rust/C unsafe binding ->
    // -----------------------------------------------------------------------
    //   Do not exit this file it will be regenerated automatically by cargo.
    //   Check:
    //     - build.rs at project root for dynamically mapping
    //     - src/capi/sockcan-map.h for static values
    // -----------------------------------------------------------------------
    ";

    let sockcan = bindgen::Builder::default()
        // main entry point for wrapper
        .header("src/capi/sockcan-map.h")
        .raw_line(header)
        // default wrapper config
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .derive_debug(false)
        .layout_tests(false)
        .allowlist_function("can_.*")
        .allowlist_function("__errno_location")
        .allowlist_function("bind")
        .allowlist_function("socket")
        .allowlist_function("setsockopt")
        .allowlist_function("ioctl")
        .allowlist_function("fcntl")
        .allowlist_function("recvfrom")
        .allowlist_function("recvmsg")
        .allowlist_function("read")
        .allowlist_function("send")
        .allowlist_function("write")
        .allowlist_function("close")
        .allowlist_function("connect")
        .allowlist_function("errno")
        .allowlist_function("strftime")
        .allowlist_function("time")
        .allowlist_function("localtime")
        .allowlist_function("strerror_r")
        .allowlist_type(".*_can")
        .allowlist_type("can_.*")
        .allowlist_type("canfd_.*")
        .allowlist_type("ifreq")
        .allowlist_type("timeval")
        .allowlist_type("bcm_msg_head")

        .blocklist_item("json_object_delete_fn")
        // generate sockcan wrapper
        .generate()
        .expect("Unable to generate sockcan");

    sockcan
        .write_to_file("src/capi/sockcan-map.rs")
        .expect("Couldn't write sockcan!");
}
