fn main() {
    cc::Build::new()
        .file("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/am_hal_ble.c")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/AmbiqMicro/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/ARM/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/")
        .compile("am_hal_ble");

    cc::Build::new()
        .file("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/am_hal_ble_patch.c")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/AmbiqMicro/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/ARM/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/")
        .compile("am_hal_ble_patch");

    cc::Build::new()
        .file("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/am_hal_ble_patch_b0.c")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/AmbiqMicro/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/ARM/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/")
        .compile("am_hal_ble_patch_b0");

    cc::Build::new()
        .file("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/am_hal_interrupt.c")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/AmbiqMicro/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/ARM/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/")
        .compile("am_hal_interrupt");

    cc::Build::new()
        .file("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/am_hal_flash.c")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/AmbiqMicro/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/CMSIS/ARM/Include/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/")
        .include("/scratch/alistair/software/mbed/AmbiqSuiteSDK/mcu/apollo3/hal/")
        .compile("am_hal_flash");
}
