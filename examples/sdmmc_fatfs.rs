#![no_main]
#![no_std]

use {
    fatfs::{IntoStorage, Write},
    log,
    stm32h7xx_hal::sdmmc::{SdCard, Sdmmc},
    stm32h7xx_hal::{pac, prelude::*, rcc},
};

#[macro_use]
mod utilities;

#[cortex_m_rt::entry]
unsafe fn main() -> ! {
    utilities::logger::init();

    // Get peripherals
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    // Constrain and Freeze power
    let pwr = dp.PWR.constrain();
    let pwrcfg = example_power!(pwr).freeze();

    // Constrain and Freeze clock
    let ccdr = dp
        .RCC
        .constrain()
        .sys_ck(200.MHz())
        .pll1_strategy(rcc::PllConfigStrategy::Iterative)
        .pll1_q_ck(100.MHz())
        .pll2_strategy(rcc::PllConfigStrategy::Iterative)
        .pll3_strategy(rcc::PllConfigStrategy::Iterative)
        .freeze(pwrcfg, &dp.SYSCFG);

    // Get the delay provider.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);

    let clk = gpioc.pc12.into_alternate().internal_pull_up(false);
    let cmd = gpiod.pd2.into_alternate().internal_pull_up(true);
    let d0 = gpioc.pc8.into_alternate().internal_pull_up(true);
    let d1 = gpioc.pc9.into_alternate().internal_pull_up(true);
    let d2 = gpioc.pc10.into_alternate().internal_pull_up(true);
    let d3 = gpioc.pc11.into_alternate().internal_pull_up(true);

    let mut sd: Sdmmc<_, SdCard> = dp.SDMMC1.sdmmc(
        (clk, cmd, d0, d1, d2, d3),
        ccdr.peripheral.SDMMC1,
        &ccdr.clocks,
    );

    // Loop until we have a card
    loop {
        // On most development boards this can be increased up to 50MHz. We choose a
        // lower frequency here so that it should work even with flying leads
        // connected to a SD card breakout.
        match sd.init(2.MHz()) {
            Ok(_) => break,
            Err(err) => {
                log::info!("Init err: {:?}", err);
            }
        }

        log::info!("Waiting for card...");

        delay.delay_ms(1000u32);
    }

    let storage = sd.into_storage();
    if let Ok(fs) = fatfs::FileSystem::new(storage, fatfs::FsOptions::new()) {
        let root_dir = fs.root_dir();

        // Write a file
        root_dir.create_dir("foo").unwrap();
        let mut file = root_dir.create_file("foo/hello.txt").unwrap();
        file.truncate().unwrap();
        file.write(b"Hello World!").unwrap();

        // Read a directory
        let dir = root_dir.open_dir("foo").unwrap();
        for r in dir.iter() {
            let entry = r.unwrap();
            log::info!("{:?}", entry.short_file_name_as_bytes());
        }
    }

    loop {
        cortex_m::asm::nop()
    }
}