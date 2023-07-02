#!/bin/sh
TRANSITION_TYPE=grow

exec swww img -o HDMI-A-1 ./HDMI-A-1.jpg --transition-type $TRANSITION_TYPE &
exec swww img -o DP-1 ./DP-1.jpg --transition-type $TRANSITION_TYPE &
exec swww img -o DVI-D-1 ./DVI-D-1.jpg --transition-type $TRANSITION_TYPE &
