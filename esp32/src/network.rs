use crate::custom_error::CustomError;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::WifiModemPeripheral;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::ipv4::IpInfo;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::EspError;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use log::info;
use std::marker::PhantomData;

pub struct Connected;
pub struct Disconnected;

pub struct Network<T> {
    ssid: String,
    password: String,
    device: BlockingWifi<EspWifi<'static>>,
    inner: PhantomData<T>,
}

impl Network<Disconnected> {
    pub fn new<M: WifiModemPeripheral, S: Into<String>>(
        modem: impl Peripheral<P = M> + 'static,
        ssid: S,
        password: S,
    ) -> Result<Network<Disconnected>, CustomError> {
        let sys_loop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take()?;

        let mut wifi =
            BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;

        Ok(Self {
            ssid: ssid.into(),
            password: password.into(),
            device: wifi,
            inner: Default::default(),
        })
    }

    pub fn connect(mut self) -> Result<Network<Connected>, CustomError> {
        let configuration = Configuration::Client(ClientConfiguration {
            ssid: self.ssid.as_str().try_into().unwrap(),
            auth_method: AuthMethod::WPA2Personal,
            password: self.password.as_str().try_into().unwrap(),
            ..Default::default()
        });

        self.device.set_configuration(&configuration)?;
        self.device.start()?;
        self.device.connect()?;
        self.device.wait_netif_up()?;

        info!("Wifi Connected");

        Ok(Network {
            ssid: self.ssid,
            password: self.password,
            device: self.device,
            inner: Default::default(),
        })
    }
}

impl Network<Connected> {
    pub fn ip_info(&self) -> Result<IpInfo, EspError> {
        self.device.wifi().sta_netif().get_ip_info()
    }

    pub fn disconnect(&mut self) -> Result<(), EspError> {
        self.device.disconnect()
    }
}
