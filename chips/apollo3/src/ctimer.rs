//! Timer driver for the Apollo3

use kernel::common::cells::OptionalCell;

use kernel::common::registers::{register_bitfields, register_structs, ReadWrite};
use kernel::common::StaticRef;
use kernel::hil;


pub static mut TIMER: Timer = Timer::new(TIMER_BASE);

const TIMER_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4000_8000 as *const TimerRegisters) };

register_structs! {
    pub TimerRegisters {
        (0x000 => tmr0: ReadWrite<u32, TMR::Register>),
        (0x004 => cmpra0: ReadWrite<u32, CMPRA::Register>),
        (0x008 => cmprb0: ReadWrite<u32, CMPRB::Register>),
        (0x00C => ctrl0: ReadWrite<u32, CTRL::Register>),
        (0x010 => _reserved0),
        (0x014 => cmprauxa0: ReadWrite<u32, CMPRAUXA::Register>),
        (0x018 => cmprauxb0: ReadWrite<u32, CMPRAUXB::Register>),
        (0x01C => aux0: ReadWrite<u32, AUX::Register>),
        (0x020 => tmr1: ReadWrite<u32, TMR::Register>),
        (0x024 => cmpra1: ReadWrite<u32, CMPRA::Register>),
        (0x028 => cmprb1: ReadWrite<u32, CMPRB::Register>),
        (0x02C => ctrl1: ReadWrite<u32, CTRL::Register>),
        (0x030 => _reserved1),
        (0x034 => cmprauxa1: ReadWrite<u32, CMPRAUXA::Register>),
        (0x038 => cmprauxb1: ReadWrite<u32, CMPRAUXB::Register>),
        (0x03C => aux1: ReadWrite<u32, AUX::Register>),
        (0x040 => tmr2: ReadWrite<u32, TMR::Register>),
        (0x044 => cmpra2: ReadWrite<u32, CMPRA::Register>),
        (0x048 => cmprb2: ReadWrite<u32, CMPRB::Register>),
        (0x04C => ctrl2: ReadWrite<u32, CTRL::Register>),
        (0x050 => _reserved2),
        (0x054 => cmprauxa2: ReadWrite<u32, CMPRAUXA::Register>),
        (0x058 => cmprauxb2: ReadWrite<u32, CMPRAUXB::Register>),
        (0x05C => aux2: ReadWrite<u32, AUX::Register>),
        (0x060 => tmr3: ReadWrite<u32, TMR::Register>),
        (0x064 => cmpra3: ReadWrite<u32, CMPRA::Register>),
        (0x068 => cmprb3: ReadWrite<u32, CMPRB::Register>),
        (0x06C => ctrl3: ReadWrite<u32, CTRL::Register>),
        (0x070 => _reserved3),
        (0x074 => cmprauxa3: ReadWrite<u32, CMPRAUXA::Register>),
        (0x078 => cmprauxb3: ReadWrite<u32, CMPRAUXB::Register>),
        (0x07C => aux3: ReadWrite<u32, AUX::Register>),
        (0x080 => tmr4: ReadWrite<u32, TMR::Register>),
        (0x084 => cmpra4: ReadWrite<u32, CMPRA::Register>),
        (0x088 => cmprb4: ReadWrite<u32, CMPRB::Register>),
        (0x08C => ctrl4: ReadWrite<u32, CTRL::Register>),
        (0x090 => _reserved4),
        (0x094 => cmprauxa4: ReadWrite<u32, CMPRAUXA::Register>),
        (0x098 => cmprauxb4: ReadWrite<u32, CMPRAUXB::Register>),
        (0x09C => aux4: ReadWrite<u32, AUX::Register>),
        (0x0A0 => tmr5: ReadWrite<u32, TMR::Register>),
        (0x0A4 => cmpra5: ReadWrite<u32, CMPRA::Register>),
        (0x0A8 => cmprb5: ReadWrite<u32, CMPRB::Register>),
        (0x0AC => ctrl5: ReadWrite<u32, CTRL::Register>),
        (0x0B0 => _reserved5),
        (0x0B4 => cmprauxa5: ReadWrite<u32, CMPRAUXA::Register>),
        (0x0B8 => cmprauxb5: ReadWrite<u32, CMPRAUXB::Register>),
        (0x0BC => aux5: ReadWrite<u32, AUX::Register>),
        (0x0C0 => tmr6: ReadWrite<u32, TMR::Register>),
        (0x0C4 => cmpra6: ReadWrite<u32, CMPRA::Register>),
        (0x0C8 => cmprb6: ReadWrite<u32, CMPRB::Register>),
        (0x0CC => ctrl6: ReadWrite<u32, CTRL::Register>),
        (0x0D0 => _reserved6),
        (0x0D4 => cmprauxa6: ReadWrite<u32, CMPRAUXA::Register>),
        (0x0D8 => cmprauxb6: ReadWrite<u32, CMPRAUXB::Register>),
        (0x0DC => aux6: ReadWrite<u32, AUX::Register>),
        (0x0a0 => tmr7: ReadWrite<u32, TMR::Register>),
        (0x0E4 => cmpra7: ReadWrite<u32, CMPRA::Register>),
        (0x0E8 => cmprb7: ReadWrite<u32, CMPRB::Register>),
        (0x0EC => ctrl7: ReadWrite<u32, CTRL::Register>),
        (0x0F0 => _reserved7),
        (0x0F4 => cmprauxa7: ReadWrite<u32, CMPRAUXA::Register>),
        (0x0F8 => cmprauxb7: ReadWrite<u32, CMPRAUXB::Register>),
        (0x0FC => aux7: ReadWrite<u32, AUX::Register>),
        (0x100 => globen: ReadWrite<u32, GLOBEN::Register>),
        (0x104 => outcfg0: ReadWrite<u32, OUTCFG::Register>),
        (0x108 => outcfg1: ReadWrite<u32, OUTCFG::Register>),
        (0x10C => outcfg2: ReadWrite<u32, OUTCFG::Register>),
        (0x110 => _reserved8),
        (0x114 => outcfg3: ReadWrite<u32, OUTCFG::Register>),
        (0x118 => incfg: ReadWrite<u32, INCFG::Register>),
        (0x11C => _reserved9),
        (0x200 => inten: ReadWrite<u32, IN::Register>),
        (0x204 => intstat: ReadWrite<u32, IN::Register>),
        (0x208 => intclr: ReadWrite<u32, IN::Register>),
        (0x20C => intset: ReadWrite<u32, IN::Register>),
        (0x210 => @END),
    }
}

register_bitfields![u32,
    TMR [
        CTTMRA OFFSET(0) NUMBITS(16) [],
        CTTMRB OFFSET(16) NUMBITS(16) []
    ],
    CMPRA [
        CMPR0A OFFSET(0) NUMBITS(16) [],
        CMPR1A OFFSET(16) NUMBITS(16) []
    ],
    CMPRB [
        CMPR0B OFFSET(0) NUMBITS(16) [],
        CMPR1B OFFSET(16) NUMBITS(16) []
    ],
    CTRL [
        TMRAEN OFFSET(0) NUMBITS(1) [],
        TMRACLK OFFSET(1) NUMBITS(5) [],
        TMRAFN OFFSET(6) NUMBITS(2) [],
        TMRAIE0 OFFSET(9) NUMBITS(1) [],
        TMRAIE1 OFFSET(10) NUMBITS(1) [],
        TMRACLR OFFSET(11) NUMBITS(1) [],
        TMRAPOL OFFSET(12) NUMBITS(1) [],
        TMRBEN OFFSET(16) NUMBITS(1) [],
        TMRBCLK OFFSET(17) NUMBITS(5) [],
        TMRBFN OFFSET(22) NUMBITS(3) [],
        TMRBIE0 OFFSET(25) NUMBITS(1) [],
        TMRBIE1 OFFSET(26) NUMBITS(1) [],
        TMRBCLR OFFSET(27) NUMBITS(1) [],
        TMRBPOL OFFSET(28) NUMBITS(1) [],
        CTLINK OFFSET(31) NUMBITS(1) []
    ],
    CMPRAUXA [
        CMPR2A OFFSET(0) NUMBITS(16) [],
        CMPR3A OFFSET(16) NUMBITS(16) []
    ],
    CMPRAUXB [
        CMPR2B OFFSET(0) NUMBITS(16) [],
        CMPR3B OFFSET(16) NUMBITS(16) []
    ],
    AUX [
        TMRALMT OFFSET(0) NUMBITS(7) []
        // TODO
    ],
    GLOBEN [
        ENA0 OFFSET(0) NUMBITS(1) [],
        ENB0 OFFSET(1) NUMBITS(1) [],
        ENA1 OFFSET(2) NUMBITS(1) [],
        ENB1 OFFSET(3) NUMBITS(1) [],
        ENA2 OFFSET(4) NUMBITS(1) [],
        ENB2 OFFSET(5) NUMBITS(1) [],
        ENA3 OFFSET(6) NUMBITS(1) [],
        ENB3 OFFSET(7) NUMBITS(1) [],
        ENA4 OFFSET(8) NUMBITS(1) [],
        ENB4 OFFSET(9) NUMBITS(1) [],
        ENA5 OFFSET(10) NUMBITS(1) [],
        ENB5 OFFSET(11) NUMBITS(1) [],
        ENA6 OFFSET(12) NUMBITS(1) [],
        ENB6 OFFSET(13) NUMBITS(1) [],
        ENA7 OFFSET(14) NUMBITS(1) [],
        ENB7 OFFSET(15) NUMBITS(1) []
    ],
    OUTCFG [
        CFG0 OFFSET(0) NUMBITS(3) [],
        CFG1 OFFSET(3) NUMBITS(3) [],
        CFG2 OFFSET(6) NUMBITS(3) [],
        CFG3 OFFSET(9) NUMBITS(3) [],
        CFG4 OFFSET(12) NUMBITS(3) [],
        CFG5 OFFSET(16) NUMBITS(3) [],
        CFG6 OFFSET(19) NUMBITS(3) [],
        CFG7 OFFSET(22) NUMBITS(3) [],
        CFG8 OFFSET(25) NUMBITS(3) [],
        CFG9 OFFSET(28) NUMBITS(3) []
    ],
    INCFG [
        CFGA0 OFFSET(0) NUMBITS(1) [],
        CFGB0 OFFSET(1) NUMBITS(1) [],
        CFGA1 OFFSET(2) NUMBITS(1) [],
        CFGB1 OFFSET(3) NUMBITS(1) [],
        CFGA2 OFFSET(4) NUMBITS(1) [],
        CFGB2 OFFSET(5) NUMBITS(1) [],
        CFGA3 OFFSET(6) NUMBITS(1) [],
        CFGB3 OFFSET(7) NUMBITS(1) [],
        CFGA4 OFFSET(8) NUMBITS(1) [],
        CFGB4 OFFSET(9) NUMBITS(1) [],
        CFGA5 OFFSET(10) NUMBITS(1) [],
        CFGB5 OFFSET(11) NUMBITS(1) [],
        CFGA6 OFFSET(12) NUMBITS(1) [],
        CFGB6 OFFSET(13) NUMBITS(1) [],
        CFGA7 OFFSET(14) NUMBITS(1) [],
        CFGB7 OFFSET(15) NUMBITS(1) []
    ],
    IN [
        CTMRA0C0INT OFFSET(0) NUMBITS(1) [],
        CTMRB0C0INT OFFSET(1) NUMBITS(1) [],
        CTMRA1C0INT OFFSET(2) NUMBITS(1) [],
        CTMRB1C0INT OFFSET(3) NUMBITS(1) [],
        CTMRA2C0INT OFFSET(4) NUMBITS(1) [],
        CTMRB2C0INT OFFSET(5) NUMBITS(1) [],
        CTMRA3C0INT OFFSET(6) NUMBITS(1) [],
        CTMRB3C0INT OFFSET(7) NUMBITS(1) [],
        CTMRA4C0INT OFFSET(8) NUMBITS(1) [],
        CTMRB4C0INT OFFSET(9) NUMBITS(1) [],
        CTMRA5C0INT OFFSET(10) NUMBITS(1) [],
        CTMRB5C0INT OFFSET(11) NUMBITS(1) [],
        CTMRA6C0INT OFFSET(12) NUMBITS(1) [],
        CTMRB6C0INT OFFSET(13) NUMBITS(1) [],
        CTMRA7C0INT OFFSET(14) NUMBITS(1) [],
        CTMRB7C0INT OFFSET(15) NUMBITS(1) [],
        CTMRA0C1INT OFFSET(16) NUMBITS(1) [],
        CTMRB0C1INT OFFSET(17) NUMBITS(1) [],
        CTMRA1C1INT OFFSET(18) NUMBITS(1) [],
        CTMRB1C1INT OFFSET(19) NUMBITS(1) [],
        CTMRA2C1INT OFFSET(20) NUMBITS(1) [],
        CTMRB2C1INT OFFSET(21) NUMBITS(1) [],
        CTMRA3C1INT OFFSET(22) NUMBITS(1) [],
        CTMRB3C1INT OFFSET(23) NUMBITS(1) [],
        CTMRA4C1INT OFFSET(24) NUMBITS(1) [],
        CTMRB4C1INT OFFSET(25) NUMBITS(1) [],
        CTMRA5C1INT OFFSET(26) NUMBITS(1) [],
        CTMRB5C1INT OFFSET(27) NUMBITS(1) [],
        CTMRA6C1INT OFFSET(28) NUMBITS(1) [],
        CTMRB6C1INT OFFSET(29) NUMBITS(1) [],
        CTMRA7C1INT OFFSET(30) NUMBITS(1) [],
        CTMRB7C1INT OFFSET(31) NUMBITS(1) []
    ]
];

pub struct Timer<'a> {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'a dyn hil::time::AlarmClient>,
}

impl Timer<'a> {
    const fn new(base: StaticRef<TimerRegisters>) -> Timer<'a> {
        Timer {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    pub fn handle_interrupt(&self) {
    }

    // starts the timer
    pub fn start(&self) {
    }
}

impl hil::time::Alarm<'a> for Timer<'a> {
    fn set_client(&self, client: &'a dyn hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: u32) {
        unimplemented!()
    }

    fn get_alarm(&self) -> u32 {
        unimplemented!()
    }

    fn disable(&self) {
        unimplemented!()
    }

    fn is_enabled(&self) -> bool {
        unimplemented!()
    }
}

impl hil::time::Time for Timer<'a> {
    type Frequency = hil::time::Freq16KHz;

    fn now(&self) -> u32 {
        unimplemented!()
    }

    fn max_tics(&self) -> u32 {
        core::u32::MAX
    }
}
