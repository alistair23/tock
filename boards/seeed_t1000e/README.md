SenseCAP Card Tracker T1000-E for Meshtastic
==================================================

<img src="https://files.seeedstudio.com/wiki/SenseCAP/Meshtastic/intro-e.png" width="35%">

The [SenseCAP Card Tracker T1000-E for Meshtastic](https://www.seeedstudio.com/SenseCAP-Card-Tracker-T1000-E-for-Meshtastic-p-5913.html) is a
board based on the Nordic nRF52840 SoC. It includes the
following:

- Semtech's LR1110
- Mediatek's AG3335 GPS

all in a credit card size sealed product.

Although the board is designed to run Meshtastic firmware, it's also capable of
running Tock.

## Getting Started

First, follow the [Tock Getting Started guide](../../doc/Getting_Started.md)

You will need Adafruit's nrfutil bootloader tool:

```shell
$ pip3 install --user adafruit-nrfutil
```

## Bootloader

The Tracker T1000-E is shipped with the Adafruit nRF52 bootloader which we use
flash the device.

### Updating the bootloader

It's likley you will need to update the bootloader as the one shipped with the
device [can be unreliable](https://meshtastic.org/docs/getting-started/flashing-firmware/nrf52/update-nrf52-bootloader/).

Follow the steps on the [Seeed wiki to update the bootloader](https://wiki.seeedstudio.com/sensecap_t1000_e/#flash-the-bootloader). Tock has been tested with the [0.9.2 bootloader](https://github.com/adafruit/Adafruit_nRF52_Bootloader/releases/tag/0.9.2).

## Programming the Kernel

### Using the Adafruit nRF52 bootloader

> **NOTE** Uploading the kernel will not change any ability to upload software to the Tracker T1000-E. The original bootloader
will not be overwrittn. All other software will work as expected.

To flash the tock-bootloader we use the Adafruit version of nRFUtil tool to communicate with the bootloader
on the board which then flashes the kernel. This requires that the bootloader be
active. To force the board into bootloader mode, press the button on the back of the board
twice in rapid succession. You should see the red LED pulse on and off.

### Flashing the Kernel

At this point you should be able to simply run `make install` in this directory
to install a fresh kernel.

https://wiki.seeedstudio.com/sensecap_t1000_e/#flash-the-bootloader
https://github.com/adafruit/Adafruit_nRF52_Bootloader/pull/336/files
https://meshtastic.discourse.group/t/building-firmware-for-rak4631/7051
https://github.com/meshtastic/firmware/blob/fb9f3610529f961c1c43464a34e002e7f0779209/variants/tracker-t1000-e/platformio.ini

rm output.u2f; uf2/utils/uf2conv.py /var/mnt/scratch/alistair/software/tock/tock/target/thumbv7em-none-eabi/release/seeed_t1000e.bin  -c -b 0x26000 -o output.u2f -f 0xADA52840; cp output.u2f /run/media/alistair/T1000-E/; sync