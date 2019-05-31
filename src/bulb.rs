use crate::refreshable_data::RefreshableData;
use failure::Error;
use lifx_core::{BuildOptions, Message, PowerLevel, RawMessage, HSBK};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

// random unique client identifier
const CLIENT_IDENTIFIER: u32 = 646_994_787;

pub enum Color {
    Single(HSBK),
    Multi(Vec<Option<HSBK>>),
}

pub struct Bulb {
    port: u32,
    target: u64,
    addr: SocketAddr,
    name: RefreshableData<String, RefreshOpts>,
    power_level: RefreshableData<PowerLevel, RefreshOpts>,
    color: RefreshableData<Color, RefreshOpts>,
    model: RefreshableData<(u32, u32), RefreshOpts>,
    location: RefreshableData<String, RefreshOpts>,
    host_firmware: RefreshableData<u32, RefreshOpts>,
    wifi_firmware: RefreshableData<u32, RefreshOpts>,
}

struct RefreshOpts {
    socket: UdpSocket,
    build_opts: BuildOptions,
    addr: SocketAddr,
}

impl Bulb {
    pub fn new(port: u32, target: u64, addr: SocketAddr) -> Self {
        let short = Duration::from_secs(15);
        let long = Duration::from_secs(5 * 60); // 5 minutes
        Self {
            port,
            target,
            addr,
            name: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
            power_level: RefreshableData::with_config(short, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
            color: RefreshableData::with_config(short, move |refresh_opts, color| match color {
                Some(Color::Single(_)) => {
                    Self::request_update(refresh_opts, Message::LightGet)
                }
                Some(Color::Multi(_)) => Self::request_update(
                    refresh_opts,
                    Message::GetColorZones {
                        start_index: 0,
                        end_index: 255,
                    },
                ),
                None => Ok(()),
            }),
            model: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
            location: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
            host_firmware: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
            wifi_firmware: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
        }
    }

    fn request_update(
        refresh_opts: &RefreshOpts,
        msg: Message,
    ) -> Result<(), Error> {
        let RefreshOpts { socket, build_opts, addr } = refresh_opts;
        socket.send_to(&RawMessage::build(build_opts, msg)?.pack()?, addr)?;

        Ok(())
    }

    pub fn check(&mut self, socket: &UdpSocket) -> Result<(), Error> {
        let build_opts = BuildOptions {
            target: Some(self.target),
            res_required: true,
            source: CLIENT_IDENTIFIER,
            ..BuildOptions::default()
        };
        let refresh_opts = RefreshOpts {
            socket: socket.try_clone().unwrap(),
            build_opts,
            addr: self.addr,
        };

        self.name.check(&refresh_opts)?;
        self.power_level.check(&refresh_opts)?;
        self.color.check(&refresh_opts)?;
        self.model.check(&refresh_opts)?;
        self.location.check(&refresh_opts)?;
        self.host_firmware.check(&refresh_opts)?;
        self.wifi_firmware.check(&refresh_opts)?;

        Ok(())
    }

    pub fn power_level(&self) -> Option<&PowerLevel> {
        self.power_level.as_ref()
    }

    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }
}
