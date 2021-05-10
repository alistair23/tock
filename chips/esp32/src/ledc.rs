//! GPIO driver.

use kernel::common::registers::interfaces::{ReadWriteable, Writeable};
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil::led::Led;
use kernel::debug;

pub const LEDC_BASE: StaticRef<LedcRegisters> =
    unsafe { StaticRef::new(0x6001_9000 as *const LedcRegisters) };

register_structs! {
    pub LedcRegisters {
        (0x00 => lsch0_conf0: ReadWrite<u32, CONF0::Register>),
        (0x04 => lsch0_hpoint: ReadWrite<u32>),
        (0x08 => lsch0_duty: ReadWrite<u32>),
        (0x0C => lsch0_conf1: ReadWrite<u32, CONF1::Register>),
        (0x10 => lsch0_duty_r: ReadWrite<u32>),
        (0x14 => lsch1_conf0: ReadWrite<u32, CONF0::Register>),
        (0x18 => lsch1_hpoint: ReadWrite<u32>),
        (0x1C => lsch1_duty: ReadWrite<u32>),
        (0x20 => lsch1_conf1: ReadWrite<u32, CONF1::Register>),
        (0x24 => lsch1_duty_r: ReadWrite<u32>),
        (0x28 => lsch2_conf0: ReadWrite<u32, CONF0::Register>),
        (0x2C => lsch2_hpoint: ReadWrite<u32>),
        (0x30 => lsch2_duty: ReadWrite<u32>),
        (0x34 => lsch2_conf1: ReadWrite<u32, CONF1::Register>),
        (0x38 => lsch2_duty_r: ReadWrite<u32>),
        (0x3C => lsch3_conf0: ReadWrite<u32, CONF0::Register>),
        (0x40 => lsch3_hpoint: ReadWrite<u32>),
        (0x44 => lsch3_duty: ReadWrite<u32>),
        (0x48 => lsch3_conf1: ReadWrite<u32, CONF1::Register>),
        (0x4C => lsch3_duty_r: ReadWrite<u32>),
        (0x50 => lsch4_conf0: ReadWrite<u32, CONF0::Register>),
        (0x54 => lsch4_hpoint: ReadWrite<u32>),
        (0x58 => lsch4_duty: ReadWrite<u32>),
        (0x5C => lsch4_conf1: ReadWrite<u32, CONF1::Register>),
        (0x60 => lsch4_duty_r: ReadWrite<u32>),
        (0x64 => lsch5_conf0: ReadWrite<u32, CONF0::Register>),
        (0x68 => lsch5_hpoint: ReadWrite<u32>),
        (0x6C => lsch5_duty: ReadWrite<u32>),
        (0x70 => lsch5_conf1: ReadWrite<u32, CONF1::Register>),
        (0x74 => lsch5_duty_r: ReadWrite<u32>),
        (0x78 => _reserved0),
        (0xa0 => lstimer0_conf: ReadWrite<u32>),
        (0xa4 => lstimer0_value: ReadWrite<u32>),
        (0xa8 => lstimer1_conf: ReadWrite<u32>),
        (0xaC => lstimer1_value: ReadWrite<u32>),
        (0xb0 => lstimer2_conf: ReadWrite<u32>),
        (0xb4 => lstimer2_value: ReadWrite<u32>),
        (0xb8 => lstimer3_conf: ReadWrite<u32>),
        (0xbc => lstimer3_value: ReadWrite<u32>),
        (0xc0 => int_raw: ReadWrite<u32>),
        (0xc4 => int_st: ReadWrite<u32>),
        (0xc8 => int_ena: ReadWrite<u32>),
        (0xcc => int_clr: ReadWrite<u32>),
        (0xd0 => conf: ReadWrite<u32>),
        (0xd4 => _reserved1),
        (0xfc => date: ReadWrite<u32>),
        (0x100 => @END),
    }
}

register_bitfields![u32,
    CONF0 [
        CLK_EN OFFSET(31) NUMBITS(1) [],
        IDLE_LV OFFSET(3) NUMBITS(1) [],
        SIG_OUT_EN OFFSET(2) NUMBITS(1) [],
    ],
    CONF1 [
        DUTY_START OFFSET(31) NUMBITS(1) [],
        DUTY_INC OFFSET(30) NUMBITS(1) [],
        DUTY_NUM OFFSET(20) NUMBITS(9) [],
        DUTY_CYCLE OFFSET(10) NUMBITS(9) [],
        DUTY_SCALE OFFSET(0) NUMBITS(9) [],
    ],
];

pub struct Ledc {
    registers: StaticRef<LedcRegisters>,
}

impl Ledc {
    pub const fn new(base: StaticRef<LedcRegisters>) -> Self {
        Self { registers: base }
    }

    pub fn handle_interrupt(&self) {
        unimplemented!()
    }
}

impl Led for Ledc {
    fn init(&self) {
        unimplemented!()
    }

    fn on(&self) {
        self.registers.lsch0_conf0.set(0xFF);
        unimplemented!()
    }

    fn off(&self) {
        debug!("Turning LED off");
        self.registers
            .lsch0_conf0
            .modify(CONF0::IDLE_LV::SET + CONF0::SIG_OUT_EN::CLEAR);
        self.registers.lsch0_conf1.modify(CONF1::DUTY_START::CLEAR);

        self.registers
            .lsch1_conf0
            .modify(CONF0::IDLE_LV::SET + CONF0::SIG_OUT_EN::CLEAR);
        self.registers.lsch1_conf1.modify(CONF1::DUTY_START::CLEAR);

        self.registers
            .lsch2_conf0
            .modify(CONF0::IDLE_LV::SET + CONF0::SIG_OUT_EN::CLEAR);
        self.registers.lsch2_conf1.modify(CONF1::DUTY_START::CLEAR);

        self.registers
            .lsch3_conf0
            .modify(CONF0::IDLE_LV::SET + CONF0::SIG_OUT_EN::CLEAR);
        self.registers.lsch3_conf1.modify(CONF1::DUTY_START::CLEAR);

        self.registers
            .lsch4_conf0
            .modify(CONF0::IDLE_LV::SET + CONF0::SIG_OUT_EN::CLEAR);
        self.registers.lsch4_conf1.modify(CONF1::DUTY_START::CLEAR);

        self.registers
            .lsch5_conf0
            .modify(CONF0::IDLE_LV::SET + CONF0::SIG_OUT_EN::CLEAR);
        self.registers.lsch5_conf1.modify(CONF1::DUTY_START::CLEAR);
    }

    fn toggle(&self) {
        unimplemented!()
    }

    fn read(&self) -> bool {
        unimplemented!()
    }
}
