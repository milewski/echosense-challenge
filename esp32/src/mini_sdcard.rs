use std::fs::File;

use esp_idf_svc::fs::littlefs::{LittleFsInfo, Littlefs};
use esp_idf_svc::hal::gpio::{AnyIOPin, InputPin, OutputPin};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::sd::spi::SdSpiHostDriver;
use esp_idf_svc::hal::sd::{SdCardConfiguration, SdCardDriver};
use esp_idf_svc::hal::spi::config::DriverConfig;
use esp_idf_svc::hal::spi::{Dma, SpiAnyPins, SpiDriver};
use esp_idf_svc::io::vfs::MountedLittlefs;
use esp_idf_svc::sys::EspError;

use crate::custom_error::CustomError;

pub struct MiniSDCard<'d> {
    filesystem: MountedLittlefs<Littlefs<SdCardDriver<SdSpiHostDriver<'d, SpiDriver<'d>>>>>,
}

impl<'d> MiniSDCard<'d> {
    pub fn new(
        spi: impl Peripheral<P = impl SpiAnyPins> + 'd,
        sck: impl Peripheral<P = impl OutputPin> + 'd,
        mosi: impl Peripheral<P = impl OutputPin> + 'd,
        miso: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
        cs: impl Peripheral<P = impl OutputPin> + 'd,
        format_on_boot: bool,
    ) -> Result<Self, CustomError> {
        let spi_driver = SpiDriver::new(
            spi,
            sck,
            mosi,
            Some(miso),
            &DriverConfig::default().dma(Dma::Auto(1024)),
        )?;

        let sd_card_driver = SdCardDriver::new_spi(
            SdSpiHostDriver::new(
                spi_driver,
                Some(cs),
                AnyIOPin::none(),
                AnyIOPin::none(),
                AnyIOPin::none(),
                None,
            )?,
            &SdCardConfiguration::new(),
        )?;

        let mut littlefs = Littlefs::new_sdcard(sd_card_driver)?;

        if format_on_boot {
            littlefs.format()?;
        }

        Ok(MiniSDCard {
            filesystem: MountedLittlefs::mount(littlefs, "/sdcard")?,
        })
    }

    pub fn info(&self) -> Result<LittleFsInfo, EspError> {
        self.filesystem.info()
    }

    pub fn create_file<T: Into<String> + AsRef<std::path::Path>>(
        &self,
        name: &T,
    ) -> Result<File, CustomError> {
        Ok(File::create(name)?)
    }
}
