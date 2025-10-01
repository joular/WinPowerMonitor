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

mod hubblo_rapl;

use std::{thread, time};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use hubblo_rapl::RaplDriver;

fn main() {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        // Change value to stop main loop
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C");

    let mut before_energy: f64 = 0.0;
    let driver = RaplDriver::new().expect("Driver failed");
    let mut first_run = true;

    while running.load(Ordering::SeqCst) {
        let after_energy = driver.get_rapl_energy().unwrap_or(0.0);
        let energy = after_energy - before_energy;
        before_energy = after_energy;

        if first_run {
            first_run = false;
            thread::sleep(time::Duration::from_secs(1));
            continue;
        }

        println!("{:.3}", energy);
        thread::sleep(time::Duration::from_secs(1));
    }
}