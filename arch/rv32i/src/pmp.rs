//! Implementation of the physical memory protection unit (PMP).

use core::cmp;
use core::fmt;

use crate::csr;
use kernel;
use kernel::common::math;
use kernel::common::registers::register_bitfields;
use kernel::mpu;

// Generic PMP config
register_bitfields![u32,
    pub pmpcfg [
        r OFFSET(0) NUMBITS(1) [],
        w OFFSET(1) NUMBITS(1) [],
        x OFFSET(2) NUMBITS(1) [],
        a OFFSET(3) NUMBITS(2) [
            OFF = 0,
            TOR = 1,
            NA4 = 2,
            NAPOT = 3
        ],
        l OFFSET(7) NUMBITS(1) []
    ]
];

/// Struct storing configuration for a RISC-V PMP region.
#[derive(Copy, Clone)]
pub struct PMPRegion {
    location: Option<(*const u8, usize)>,
    base_address: u32,
    cfg: tock_registers::registers::FieldValue<u32, pmpcfg::Register>,
}

impl PMPRegion {
    fn new(
        start: *const u8,
        base_address: u32,
        size: usize,
        permissions: mpu::Permissions,
    ) -> PMPRegion {
        // Determine access and execute permissions
        let pmpcfg = match permissions {
            mpu::Permissions::ReadWriteExecute => {
                pmpcfg::r::SET + pmpcfg::w::SET + pmpcfg::x::SET + pmpcfg::a::NAPOT
            }
            mpu::Permissions::ReadWriteOnly => {
                pmpcfg::r::SET + pmpcfg::w::SET + pmpcfg::x::CLEAR + pmpcfg::a::NAPOT
            }
            mpu::Permissions::ReadExecuteOnly => {
                pmpcfg::r::SET + pmpcfg::w::CLEAR + pmpcfg::x::SET + pmpcfg::a::NAPOT
            }
            mpu::Permissions::ReadOnly => {
                pmpcfg::r::SET + pmpcfg::w::CLEAR + pmpcfg::x::CLEAR + pmpcfg::a::NAPOT
            }
            mpu::Permissions::ExecuteOnly => {
                pmpcfg::r::CLEAR + pmpcfg::w::CLEAR + pmpcfg::x::SET + pmpcfg::a::NAPOT
            }
        };

        PMPRegion {
            location: Some((start, size)),
            base_address: base_address,
            cfg: pmpcfg,
        }
    }

    fn empty(_region_num: usize) -> PMPRegion {
        PMPRegion {
            location: None,
            base_address: 0,
            cfg: pmpcfg::r::CLEAR + pmpcfg::w::CLEAR + pmpcfg::x::CLEAR,
        }
    }

    fn location(&self) -> Option<(*const u8, usize)> {
        self.location
    }

    fn overlaps(&self, other_start: *const u8, other_size: usize) -> bool {
        let other_start = other_start as usize;
        let other_end = other_start + other_size;

        let (region_start, region_end) = match self.location {
            Some((region_start, region_size)) => {
                let region_start = region_start as usize;
                let region_end = region_start + region_size;
                (region_start, region_end)
            }
            None => return false,
        };

        if region_start < other_end && other_start < region_end {
            true
        } else {
            false
        }
    }
}

/// Struct storing region configuration for RISCV PMP.
#[derive(Copy, Clone)]
pub struct PMPConfig {
    regions: [PMPRegion; 16],
    total_regions: usize,
}

const APP_MEMORY_REGION_NUM: usize = 0;

impl Default for PMPConfig {
    /// number of regions on the arty chip
    fn default() -> PMPConfig {
        PMPConfig {
            regions: [
                PMPRegion::empty(0),
                PMPRegion::empty(1),
                PMPRegion::empty(2),
                PMPRegion::empty(3),
                PMPRegion::empty(4),
                PMPRegion::empty(5),
                PMPRegion::empty(6),
                PMPRegion::empty(7),
                PMPRegion::empty(8),
                PMPRegion::empty(9),
                PMPRegion::empty(10),
                PMPRegion::empty(11),
                PMPRegion::empty(12),
                PMPRegion::empty(13),
                PMPRegion::empty(14),
                PMPRegion::empty(15),
            ],
            total_regions: 16,
        }
    }
}

impl fmt::Display for PMPConfig {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl PMPConfig {
    pub fn new(total_regions: usize) -> PMPConfig {
        if total_regions > 16 {
            panic!("There is an ISA maximum of 16 PMP regions");
        }
        if total_regions < 4 {
            panic!("Tock requires at least 4 PMP regions");
        }
        PMPConfig {
            regions: [
                PMPRegion::empty(0),
                PMPRegion::empty(1),
                PMPRegion::empty(2),
                PMPRegion::empty(3),
                PMPRegion::empty(4),
                PMPRegion::empty(5),
                PMPRegion::empty(6),
                PMPRegion::empty(7),
                PMPRegion::empty(8),
                PMPRegion::empty(9),
                PMPRegion::empty(10),
                PMPRegion::empty(11),
                PMPRegion::empty(12),
                PMPRegion::empty(13),
                PMPRegion::empty(14),
                PMPRegion::empty(15),
            ],
            total_regions: total_regions,
        }
    }

    fn unused_region_number(&self) -> Option<usize> {
        for (number, region) in self.regions.iter().enumerate() {
            if number == APP_MEMORY_REGION_NUM {
                continue;
            }
            if let None = region.location() {
                if number < self.total_regions {
                    return Some(number);
                }
            }
        }
        None
    }
}

impl kernel::mpu::MPU for PMPConfig {
    type MpuConfig = PMPConfig;

    fn enable_mpu(&self) {}

    fn disable_mpu(&self) {
        for x in 0..16 {
            // If PMP is supported by the core then all 16 register sets must exist
            // They don't all have to do anything, but let's zero them all just in case.
            match x {
                0 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr0.set(0x0);
                }
                1 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr1.set(0x0);
                }
                2 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr2.set(0x0);
                }
                3 => {
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
                    csr::CSR.pmpaddr3.set(0x0);
                }
                4 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr4.set(0x0);
                }
                5 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr5.set(0x0);
                }
                6 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr6.set(0x0);
                }
                7 => {
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg1.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
                    csr::CSR.pmpaddr7.set(0x0);
                }
                8 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr8.set(0x0);
                }
                9 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr9.set(0x0);
                }
                10 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr10.set(0x0);
                }
                11 => {
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg2.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
                    csr::CSR.pmpaddr11.set(0x0);
                }
                12 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r0::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w0::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x0::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a0::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l0::CLEAR);
                    csr::CSR.pmpaddr12.set(0x0);
                }
                13 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r1::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w1::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x1::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a1::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l1::CLEAR);
                    csr::CSR.pmpaddr13.set(0x0);
                }
                14 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r2::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w2::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x2::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a2::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l2::CLEAR);
                    csr::CSR.pmpaddr14.set(0x0);
                }
                15 => {
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::r3::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::w3::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::x3::CLEAR);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::a3::OFF);
                    csr::CSR.pmpcfg3.modify(csr::pmpconfig::pmpcfg::l3::CLEAR);
                    csr::CSR.pmpaddr15.set(0x0);
                }
                // spec 1.10 only goes to 15
                _ => break,
            }
        }
        //set first PMP to have permissions to entire space
        csr::CSR.pmpaddr0.set(0xFFFF_FFFF);
        //enable R W X fields
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::r0::SET);
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::w0::SET);
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::x0::SET);
        csr::CSR.pmpcfg0.modify(csr::pmpconfig::pmpcfg::a0::OFF)
    }

    fn number_total_regions(&self) -> usize {
        self.total_regions
    }

    fn allocate_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_region_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<mpu::Region> {
        for region in config.regions.iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

        let region_num = config.unused_region_number()?;

        // Logical region
        let mut start = unallocated_memory_start as usize;
        let mut size = min_region_size;

        // Region start always has to align to the size
        if start % size != 0 {
            start += size - (start % size);
        }

        // Regions must be at least 8 bytes
        if size < 8 {
            size = 8;
        }

        let shift = math::log_base_two(size as u32) - 2;
        let mask = (1 << shift) - 1;
        let base_address = (((start as u32) >> 2) & !mask) | (mask >> 1);

        let region = PMPRegion::new(start as *const u8, base_address, size, permissions);

        config.regions[region_num] = region;

        Some(mpu::Region::new(start as *const u8, size))
    }

    fn allocate_app_memory_region(
        &self,
        unallocated_memory_start: *const u8,
        unallocated_memory_size: usize,
        min_memory_size: usize,
        initial_app_memory_size: usize,
        initial_kernel_memory_size: usize,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Option<(*const u8, usize)> {
        // Check that no previously allocated regions overlap the unallocated memory.
        for region in config.regions.iter() {
            if region.overlaps(unallocated_memory_start, unallocated_memory_size) {
                return None;
            }
        }

        // Make sure there is enough memory for app memory and kernel memory.
        let memory_size = cmp::max(
            min_memory_size,
            initial_app_memory_size + initial_kernel_memory_size,
        );

        let region_size = math::closest_power_of_two(memory_size as u32) as usize;

        // The region should start as close as possible to the start of the unallocated memory.
        let mut region_start = unallocated_memory_start as usize;

        // Region start always has to align to the size
        if region_start % region_size != 0 {
            region_start += region_size - (region_start % region_size);
        }

        // Make sure the region fits in the unallocated memory.
        if region_start + region_size
            > (unallocated_memory_start as usize) + unallocated_memory_size
        {
            return None;
        }

        debug!("2 region_start: 0x{:x}; region_size: 0x{:x}", region_start, region_size);

        let shift = math::log_base_two(region_size as u32) - 2;
        let mask = (1 << shift) - 1;
        let base_address = (((region_start as u32) >> 2) & !mask) | (mask >> 1);

        let region = PMPRegion::new(
            region_start as *const u8,
            base_address,
            region_size,
            permissions,
        );

        config.regions[APP_MEMORY_REGION_NUM] = region;

        Some((region_start as *const u8, region_size))
    }

    fn update_app_memory_region(
        &self,
        app_memory_break: *const u8,
        kernel_memory_break: *const u8,
        permissions: mpu::Permissions,
        config: &mut Self::MpuConfig,
    ) -> Result<(), ()> {
        let (region_start, region_size) = match config.regions[APP_MEMORY_REGION_NUM].location() {
            Some((start, size)) => (start as usize, size),
            None => {
                // Error: Process tried to update app memory MPU region before it was created.
                return Err(());
            }
        };

        let app_memory_break = app_memory_break as usize;
        let kernel_memory_break = kernel_memory_break as usize;

        // Region start always has to align to the size
        if region_start % region_size != 0 {
            region_start += region_size - (region_start % region_size);
        }

        // Out of memory
        if app_memory_break > kernel_memory_break {
            return Err(());
        }

        let shift = math::log_base_two(region_size as u32) - 2;
        let mask = (1 << shift) - 1;
        let base_address = (((region_start as u32) >> 2) & !mask) | (mask >> 1);

        let region = PMPRegion::new(
            region_start as *const u8,
            base_address,
            region_size,
            permissions,
        );

        config.regions[APP_MEMORY_REGION_NUM] = region;

        Ok(())
    }

    fn configure_mpu(&self, config: &Self::MpuConfig) {
        // Clear the pmpcfg0 register as this is set by the disable function
        csr::CSR.pmpcfg0.set(0);

        for x in 0..self.total_regions {
            let region = config.regions[x];
            let cfg_val = region.cfg.value << ((x % 4) * 8);

            match x {
                0 => {
                    csr::CSR.pmpcfg0.set(cfg_val | csr::CSR.pmpcfg0.get());
                    csr::CSR.pmpaddr0.set(region.base_address);
                }
                1 => {
                    csr::CSR.pmpcfg0.set(cfg_val | csr::CSR.pmpcfg0.get());
                    csr::CSR.pmpaddr1.set(region.base_address);
                }
                2 => {
                    csr::CSR.pmpcfg0.set(cfg_val | csr::CSR.pmpcfg0.get());
                    csr::CSR.pmpaddr2.set(region.base_address);
                }
                3 => {
                    csr::CSR.pmpcfg0.set(cfg_val | csr::CSR.pmpcfg0.get());
                    csr::CSR.pmpaddr3.set(region.base_address);
                }
                4 => {
                    csr::CSR.pmpcfg1.set(cfg_val | csr::CSR.pmpcfg1.get());
                    csr::CSR.pmpaddr4.set(region.base_address);
                }
                5 => {
                    csr::CSR.pmpcfg1.set(cfg_val | csr::CSR.pmpcfg1.get());
                    csr::CSR.pmpaddr5.set(region.base_address);
                }
                6 => {
                    csr::CSR.pmpcfg1.set(cfg_val | csr::CSR.pmpcfg1.get());
                    csr::CSR.pmpaddr6.set(region.base_address);
                }
                7 => {
                    csr::CSR.pmpcfg1.set(cfg_val | csr::CSR.pmpcfg1.get());
                    csr::CSR.pmpaddr7.set(region.base_address);
                }
                8 => {
                    csr::CSR.pmpcfg2.set(cfg_val | csr::CSR.pmpcfg2.get());
                    csr::CSR.pmpaddr8.set(region.base_address);
                }
                9 => {
                    csr::CSR.pmpcfg2.set(cfg_val | csr::CSR.pmpcfg2.get());
                    csr::CSR.pmpaddr9.set(region.base_address);
                }
                10 => {
                    csr::CSR.pmpcfg2.set(cfg_val | csr::CSR.pmpcfg2.get());
                    csr::CSR.pmpaddr10.set(region.base_address);
                }
                11 => {
                    csr::CSR.pmpcfg2.set(cfg_val | csr::CSR.pmpcfg2.get());
                    csr::CSR.pmpaddr11.set(region.base_address);
                }
                12 => {
                    csr::CSR.pmpcfg3.set(cfg_val | csr::CSR.pmpcfg3.get());
                    csr::CSR.pmpaddr12.set(region.base_address);
                }
                13 => {
                    csr::CSR.pmpcfg3.set(cfg_val | csr::CSR.pmpcfg3.get());
                    csr::CSR.pmpaddr13.set(region.base_address);
                }
                14 => {
                    csr::CSR.pmpcfg3.set(cfg_val | csr::CSR.pmpcfg3.get());
                    csr::CSR.pmpaddr14.set(region.base_address);
                }
                15 => {
                    csr::CSR.pmpcfg3.set(cfg_val | csr::CSR.pmpcfg3.get());
                    csr::CSR.pmpaddr15.set(region.base_address);
                }
                _ => break,
            }
        }
    }
}
