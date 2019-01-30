#! /bin/bash

cargo build --release || exit -1

export BINFILE="target/thumbv7em-none-eabihf/release/nordic-eink"
# export BINFILE  "target/thumbv7em-none-eabihf/debug/nordic-eink"

export WAITING='Waiting for media'
while [[ ! -w /media/$USER/JLINK/ ]] ;
do
    echo -n $WAITING
    sleep 2;
    export WAITING=''
done
install -d docs/release
echo arm-none-eabi-objcopy $BINFILE /media/$USER/JLINK/out.hex -O ihex
arm-none-eabi-objcopy $BINFILE  docs/release/out.hex  -O ihex
cp docs/release/out.hex /media/$USER/JLINK/
