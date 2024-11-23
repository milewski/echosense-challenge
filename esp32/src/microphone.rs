use std::io::Write;

use esp_idf_svc::hal::gpio::{AnyIOPin, InputPin, OutputPin};
use esp_idf_svc::hal::i2s::config::SlotMode::Mono;
use esp_idf_svc::hal::i2s::config::{
    ClockSource, Config, DataBitWidth, MclkMultiple, StdClkConfig, StdConfig, StdGpioConfig,
    StdSlotConfig,
};
use esp_idf_svc::hal::i2s::{I2sDriver, I2sRx};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::io::Read;
use esp_idf_svc::sys::EspError;
use crate::custom_error::CustomError;

pub struct Microphone<'d, T, const BUFFER_SIZE: usize> {
    device: I2sDriver<'d, T>,
    buffer: [u8; BUFFER_SIZE],
}

impl<'d, const BUFFER_SIZE: usize> Microphone<'d, I2sRx, BUFFER_SIZE> {
    pub fn new<I2S: esp_idf_svc::hal::i2s::I2s>(
        i2s: impl Peripheral<P=I2S> + 'd,
        sck: impl Peripheral<P=impl OutputPin + InputPin> + 'd,
        sd: impl Peripheral<P=impl InputPin> + 'd,
        ws: impl Peripheral<P=impl OutputPin + InputPin> + 'd,
        sample_rate_hz: u32,
    ) -> Result<Self, CustomError> {
        let channel_config = Config::new()
            .dma_buffer_count(2)
            .frames_per_buffer((BUFFER_SIZE * 2) as u32);

        let clock_config =
            StdClkConfig::new(sample_rate_hz, ClockSource::Pll160M, MclkMultiple::M512);
        let slot_config = StdSlotConfig::philips_slot_default(DataBitWidth::Bits16, Mono);
        let gpio_cfg = StdGpioConfig::new(false, false, false);
        let i2s_std_config = StdConfig::new(channel_config, clock_config, slot_config, gpio_cfg);

        let mut i2s = I2sDriver::new_std_rx(i2s, &i2s_std_config, sck, sd, None::<AnyIOPin>, ws)?;

        i2s.rx_enable()?;

        Ok(Microphone {
            device: i2s,
            buffer: [0; BUFFER_SIZE],
        })
    }

    pub fn sample(&mut self) -> Result<&[u8], CustomError> {
        self.device.read_exact(&mut self.buffer)?;

        Ok(self.buffer.as_slice())
    }

    pub fn start(&mut self) -> Result<(), EspError> {
        self.device.rx_enable()
    }

    pub fn stop(&mut self) -> Result<(), EspError> {
        self.device.rx_disable()
    }
}
