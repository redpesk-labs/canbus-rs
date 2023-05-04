## Start demo

* start virtual can injection
    * apt-get install can-utils;  dnf install can-utils; zypper install can-utils;
    * canplayer vcan0=elmcan -v -I ./example/bms/etc/candump/BMS.log -l i -g 10

* start can-bms
```
[fulup@fulup-laptop canbus-rs]$ ~/.cargo/build/debug/can-bms vcan0 500
(1) => CanID:545 opcode:RxChanged stamp:1683190973981718
  -- RemainingRunTime          value:0         (u8) status:Unchanged age:0 json:
    {"status":"Unchanged","name":"RemainingRunTime","stamp":0,"value":0}
  -- MinCellVoltage            value:2976     (u16) status:Updated age:0 json:
    {"status":"Updated","name":"MinCellVoltage","stamp":1683190973981718,"value":2976}
  -- MaxCellVoltage            value:3000     (u16) status:Updated age:0 json:
    {"status":"Updated","name":"MaxCellVoltage","stamp":1683190973981718,"value":3000}
  -- BatVoltage                value:671      (u16) status:Updated age:0 json:
    {"status":"Updated","name":"BatVoltage","stamp":1683190973981718,"value":671}
  -- BatSoc                    value:1         (u8) status:Updated age:0 json:
    {"status":"Updated","name":"BatSoc","stamp":1683190973981718,"value":1}
  ...
```