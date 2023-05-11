#!/bin/bash

DIRNAME=`dirname $0`
canplayer vcan0=elmcan -v -l i -g 10 -I $DIRNAME/candump/model3.log
