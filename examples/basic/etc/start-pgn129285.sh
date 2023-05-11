#!/bin/bash

DIRNAME=`dirname $0`
canplayer -v -l i -g 10 -I $DIRNAME/candump/j1939-pgn129285.log
