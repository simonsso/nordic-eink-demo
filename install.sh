#! /bin/bash

cargo build || exit -1

export WAITING='Waiting for media'
while [[ ! -w /media/$USER/JLINK/ ]] ;
do
    echo -n $WAITING
    sleep 2;
    export WAITING=''
done
install -d docs/release
echo arm-none-eabi-objcopy target/thumbv6m-none-eabi/debug/microbit /media/$USER/JLINK/out.hex -O ihex
arm-none-eabi-objcopy target/thumbv7em-none-eabihf/debug/nordic-eink  docs/release/out.hex  -O ihex
cp docs/release/out.hex /media/$USER/JLINK/
