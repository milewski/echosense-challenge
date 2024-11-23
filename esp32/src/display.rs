use crate::custom_error::CustomError;
use crate::images::dev_to_logo::DEV_TO_LOGO;
use crate::images::wifi_icon::WIFI_ICON;
use crate::qrcode::QRCode;
use embedded_graphics::image::{Image, ImageRaw};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::Point;
use embedded_graphics::Drawable;
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::i2c::config::Config;
use esp_idf_svc::hal::i2c::{I2c, I2cDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use crate::images::done_icon::DONE_ICON;

pub struct Display<'d> {
    state: DrawState,
    driver: Ssd1306<I2CInterface<I2cDriver<'d>>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
}

#[derive(PartialEq)]
pub enum DrawState {
    Initializing,
    Wifi,
    Done,
    QRCode(String),
}

impl<'d> Display<'d> {
    pub fn new<I2C: I2c>(
        i2c: impl Peripheral<P=I2C> + 'd,
        sda: impl Peripheral<P=impl InputPin + OutputPin> + 'd,
        scl: impl Peripheral<P=impl InputPin + OutputPin> + 'd,
    ) -> Result<Self, CustomError> {
        let i2c = I2cDriver::new(
            i2c,
            sda,
            scl,
            &Config::default(),
        )?;

        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();

        display.init()?;

        Ok(Self {
            state: DrawState::Wifi,
            driver: display,
        })
    }

    pub fn draw(&mut self, state: DrawState) -> Result<(), CustomError> {
        if self.state == state {
            return Ok(());
        }

        let byte = match &state {
            DrawState::Initializing => DEV_TO_LOGO.to_vec(),
            DrawState::Wifi => WIFI_ICON.to_vec(),
            DrawState::Done => DONE_ICON.to_vec(),
            DrawState::QRCode(content) => QRCode::new(content, 128, 64, 2, (20, 3))?.to_vec(),
        };

        self.driver.clear_buffer();

        let raw: ImageRaw<BinaryColor> = ImageRaw::new(&byte, 128);
        let image = Image::new(&raw, Point::new(0, 0));

        image.draw(&mut self.driver)?;

        self.driver.flush()?;

        self.state = state;

        Ok(())
    }
}