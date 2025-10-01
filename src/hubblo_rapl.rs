/*
 * Copyright (c) 2025, Adel Noureddine, Université Paris Nanterre.
 * All rights reserved. This program and the accompanying materials
 * are made available under the terms of the
 * GNU General Public License v3.0 only (GPL-3.0-only)
 * which accompanies this distribution, and is available at
 * https://www.gnu.org/licenses/gpl-3.0.en.html
 *
 * Author : Adel Noureddine
 */

use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::IO::DeviceIoControl;

// RAPL MSR addresses
const MSR_RAPL_POWER_UNIT: u64 = 0x606;
const MSR_PKG_ENERGY_STATUS: u64 = 0x611;
const MSR_DRAM_ENERGY_STATUS: u64 = 0x619;
const MSR_PLATFORM_ENERGY_STATUS: u64 = 0x64d;

// CTL_CODE macro implementation
const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}

const FILE_DEVICE_UNKNOWN: u32 = 0x00000022;
const METHOD_BUFFERED: u32 = 0;
const FILE_READ_DATA: u32 = 0x0001;
const FILE_WRITE_DATA: u32 = 0x0002;

pub struct RaplDriver {
    handle: HANDLE,
    power_unit: f64,
    energy_unit: f64,
    time_unit: f64,
    psys: bool,
    pkg: bool,
    dram: bool,
}

impl RaplDriver {
    pub fn new() -> Result<Self> {
        let driver_path = w!("\\\\.\\ScaphandreDriver");

        let handle = unsafe {
            CreateFileW(
                driver_path,
                FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED,
                None,
            )?
        };

        if handle == INVALID_HANDLE_VALUE {
            return Err(Error::from_thread());
        }

        let mut driver = RaplDriver {
            handle,
            power_unit: 0.0,
            energy_unit: 0.0,
            time_unit: 0.0,
            psys: false,
            pkg: false,
            dram: false,
        };

        driver.get_energy_units()?;
        driver.check_supported_platform()?;

        Ok(driver)
    }

    fn get_rapl_ctl_code() -> u32 {
        ctl_code(
            FILE_DEVICE_UNKNOWN,
            MSR_RAPL_POWER_UNIT as u32,
            METHOD_BUFFERED,
            FILE_READ_DATA | FILE_WRITE_DATA,
        )
    }

    fn get_data_from_driver(&self, msr: u64) -> Result<u64> {
        let mut reply_data: u64 = 0;
        let mut bytes_returned: u32 = 0;

        let ctl_code = Self::get_rapl_ctl_code();

        unsafe {
            DeviceIoControl(
                self.handle,
                ctl_code,
                Some(&msr as *const u64 as *const _),
                std::mem::size_of::<u64>() as u32,
                Some(&mut reply_data as *mut u64 as *mut _),
                std::mem::size_of::<u64>() as u32,
                Some(&mut bytes_returned),
                None,
            )?;
        }

        Ok(reply_data)
    }

    fn get_energy_units(&mut self) -> Result<()> {
        let reply_data = self.get_data_from_driver(MSR_RAPL_POWER_UNIT)?;

        // Time Units
        const TIME_MASK: u64 = 0xF0000;
        let time_val = reply_data & TIME_MASK;
        self.time_unit = 1.0 / 2.0_f64.powi((time_val >> 16) as i32);

        // Energy Units
        const ENERGY_MASK: u64 = 0x1F00;
        let energy_val = reply_data & ENERGY_MASK;
        self.energy_unit = 1.0 / 2.0_f64.powi((energy_val >> 8) as i32);

        // Power Units
        const POWER_MASK: u64 = 0xF;
        let power_val = reply_data & POWER_MASK;
        self.power_unit = 1.0 / 2.0_f64.powi(power_val as i32);

        Ok(())
    }

    fn check_supported_platform(&mut self) -> Result<()> {
        // Check for PSYS (Platform) support
        if let Ok(reply_data) = self.get_data_from_driver(MSR_PLATFORM_ENERGY_STATUS) {
            if reply_data != 0 {
                self.psys = true;
                return Ok(());
            }
        }

        // If PSYS not supported, check PKG and DRAM
        if let Ok(reply_data) = self.get_data_from_driver(MSR_PKG_ENERGY_STATUS) {
            if reply_data != 0 {
                self.pkg = true;
            }
        }

        if let Ok(reply_data) = self.get_data_from_driver(MSR_DRAM_ENERGY_STATUS) {
            if reply_data != 0 {
                self.dram = true;
            }
        }

        Ok(())
    }

    pub fn get_rapl_energy(&self) -> Result<f64> {
        if self.psys {
            let reply_data = self.get_data_from_driver(MSR_PLATFORM_ENERGY_STATUS)?;
            let raw_psys_energy = (reply_data & 0xFFFFFFFF) as u32;
            let psys_energy = raw_psys_energy as f64 * self.energy_unit;
            return Ok(psys_energy);
        }

        if self.pkg {
            let reply_data = self.get_data_from_driver(MSR_PKG_ENERGY_STATUS)?;
            let raw_pkg_energy = (reply_data & 0xFFFFFFFF) as u32;
            let pkg_energy = raw_pkg_energy as f64 * self.energy_unit;

            if self.dram {
                let reply_data = self.get_data_from_driver(MSR_DRAM_ENERGY_STATUS)?;
                let raw_dram_energy = (reply_data & 0xFFFFFFFF) as u32;
                let dram_energy = raw_dram_energy as f64 * self.energy_unit;
                return Ok(pkg_energy + dram_energy);
            }

            return Ok(pkg_energy);
        }

        Ok(0.0)
    }
}

impl Drop for RaplDriver {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}