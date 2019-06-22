use crate::refreshable_data::RefreshableData;
use chrono::prelude::*;
use failure::Error;
use lifx_core::{BuildOptions, Message, PowerLevel, RawMessage, HSBK};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

// random unique client identifier
const CLIENT_IDENTIFIER: u32 = 646_994_787;

#[derive(Debug)]
pub enum Color {
    Single(Option<HSBK>),
    Multi(Vec<Option<HSBK>>),
}

#[derive(Debug)]
pub struct Location {
    id: [u8; 16],
    name: String,
    updated_at: DateTime<Utc>,
}

impl Location {
    pub fn new(id: [u8; 16], name: String, updated_at: DateTime<Utc>) -> Self {
        Self {
            id,
            name,
            updated_at,
        }
    }
}

#[derive(Debug)]
pub struct Bulb {
    pub(crate) addr: SocketAddr,
    pub(crate) target: u64,
    pub(crate) name: RefreshableData<String, RefreshOpts>,
    pub(crate) power: RefreshableData<PowerLevel, RefreshOpts>,
    pub(crate) color: RefreshableData<Color, RefreshOpts>,
    pub(crate) model: RefreshableData<(u32, u32), RefreshOpts>,
    pub(crate) location: RefreshableData<Location, RefreshOpts>,
    pub(crate) host_firmware: RefreshableData<u32, RefreshOpts>,
    pub(crate) wifi_firmware: RefreshableData<u32, RefreshOpts>,
}

pub(crate) struct RefreshOpts {
    socket: UdpSocket,
    build_opts: BuildOptions,
    addr: SocketAddr,
}

impl Bulb {
    pub fn new(addr: SocketAddr, target: u64) -> Self {
        let short = Duration::from_secs(15);
        let long = Duration::from_secs(5 * 60); // 5 minutes
        Self {
            addr,
            target,
            name: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
            power: RefreshableData::with_config(short, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetPower)
            }),
            color: RefreshableData::with_config(short, move |refresh_opts, color| match color {
                Some(Color::Single(_)) => Self::request_update(refresh_opts, Message::LightGet),
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
                Self::request_update(refresh_opts, Message::GetVersion)
            }),
            location: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetLocation)
            }),
            host_firmware: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetHostFirmware)
            }),
            wifi_firmware: RefreshableData::with_config(long, move |refresh_opts, _| {
                Self::request_update(refresh_opts, Message::GetWifiFirmware)
            }),
        }
    }

    fn request_update(refresh_opts: &RefreshOpts, msg: Message) -> Result<(), Error> {
        let RefreshOpts {
            socket,
            build_opts,
            addr,
        } = refresh_opts;
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
        self.power.check(&refresh_opts)?;
        self.color.check(&refresh_opts)?;
        self.model.check(&refresh_opts)?;
        self.location.check(&refresh_opts)?;
        self.host_firmware.check(&refresh_opts)?;
        self.wifi_firmware.check(&refresh_opts)?;

        Ok(())
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn power(&self) -> Option<&PowerLevel> {
        self.power.as_ref()
    }

    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    pub fn color_single(&self) -> Option<&HSBK> {
        self.color.as_ref().and_then(|color| match color {
            Color::Single(hsbk) => hsbk.as_ref(),
            _ => None,
        })
    }

    pub fn color_multi(&self) -> Option<&Vec<Option<HSBK>>> {
        self.color.as_ref().and_then(|color| match color {
            Color::Multi(hsbk) => Some(hsbk),
            _ => None,
        })
    }
}
