/*
 * Copyright (C) 2015-2023 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Redpesk samples code/config use MIT License and can be freely copy/modified even within proprietary code
 * License: $RP_BEGIN_LICENSE$ SPDX:MIT https://opensource.org/licenses/MIT $RP_END_LICENSE$
 */

#![doc(
    html_logo_url = "https://iot.bzh/images/defaults/company/512-479-max-transp.png",
    html_favicon_url = "https://iot.bzh/images/defaults/favicon.ico"
)]
extern crate sockcan;
extern crate libafb;

// import libafb dependencies
libafb::AfbModImport!();

// This rootv4 demonstrate how to test an external rootv4 that you load within the same afb-binder process and security context
// It leverages test (Test Anything Protocol) that is compatible with redpesk testing report.
struct TapUserData {
    autostart: bool,
    autoexit: bool,
    output: AfbTapOutput,
}

// AfbApi userdata implements AfbApiControls trait
impl AfbApiControls for TapUserData {
    fn start(&mut self, api: &AfbApi) -> i32 {
        afb_log_msg!(Notice, api, "starting TAP testing");

        // ------ Simple verb -----------
        let ping =
            AfbTapTest::new("sockcan-ping", "low-can", "ping").set_info("My simple ping test");

        let dbc_simple = AfbTapTest::new("sockcan-dbc-simple", "low-can", "dbc/parse")
            .set_info("Parse a simple DBC file")
            .add_arg(&JsonStr(
                "{'input':'examples/etc/sample.dbc', 'output':'none'}",
            ))
            .expect("valid json");

        let dbc_model3 = AfbTapTest::new("sockcan-dbc-simple", "low-can", "dbc/parse")
            .set_info("Parse a simple DBC file")
            .add_arg(&JsonStr(
                "{'input':'examples/etc/model3can.dbc', 'output':'none'}",
            ))
            .expect("valid json");


        let test_suite = AfbTapSuite::new(api, "sockcan Apis Test")
            .set_info("Rust low can Api TAP test")
            .add_test(ping)
            .add_test(dbc_simple)
            .add_test(dbc_model3)
            .set_autorun(self.autostart)
            .set_autoexit(self.autoexit)
            .set_output(self.output)
            .finalize();

        match test_suite {
            Err(Error) => {
                afb_log_msg!(Critical, api, "Tap test fail to start Error={}", Error);
                AFB_FATAL
            }
            Ok(_json) => AFB_OK,
        }
    }

    fn config(&mut self, api: &AfbApi, jconf: AfbJsonObj) -> i32 {
        afb_log_msg!(Debug, api, "api={} config={}", api.get_uid(), jconf);
        match jconf.get::<bool>("autostart") {
            Ok(value) => self.autostart = value,
            Err(_Error) => {}
        };

        match jconf.get::<bool>("autoexit") {
            Ok(value) => self.autoexit = value,
            Err(_Error) => {}
        };

        match jconf.get::<String>("output") {
            Err(_Error) => {}
            Ok(value) => match value.to_uppercase().as_str() {
                "JSON" => self.output = AfbTapOutput::JSON,
                "TAP" => self.output = AfbTapOutput::TAP,
                "NONE" => self.output = AfbTapOutput::NONE,
                _ => {
                    afb_log_msg!(
                        Error,
                        api,
                        "Invalid output should be json|tap (default used)"
                    );
                }
            },
        };

        AFB_OK
    }

    // mandatory for downcasting back to custom apidata object
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

// rootv4 init callback started at rootv4 load time before any API exist
// -----------------------------------------
pub fn binding_test_init(rootv4: AfbApiV4, jconf: sockcanObj) -> i32 {
    let uid = match jconf.get::<String>("uid") {
        Ok(value) => value,
        Err(_Error) => "Tap-test-rootv4".to_owned(),
    };

    let tap_config = TapUserData {
        autostart: true,
        autoexit: true,
        output: AfbTapOutput::TAP,
    };


    afb_log_msg!(Notice, rootv4, "-- rootv4 {} loaded", uid);
    match AfbApi::new("sockcan-test")
        .set_info("Tap Low Can reporting")
        .require_api("low-can")
        .set_callback(Box::new(tap_config))
        .seal(false)
        .finalize()
    {
        Ok(api) => {
            afb_log_msg!(Notice, rootv4, "Tap test starting uid={}", api.get_uid());
            AFB_OK
        }
        Err(Error) => {
            afb_log_msg!(Critical, rootv4, "Fail to register api Error={}", Error);
            AFB_FATAL
        }
    }
}

// register rootv4 within libafb
AfbBindingRegister!(binding_test_init);
