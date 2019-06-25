use crate::refreshable_data::RefreshableData;
use chrono::prelude::*;
use failure::Error;
use lifx_core::{BuildOptions, Message, PowerLevel, RawMessage, HSBK};
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;

#[derive(Clone, Debug)]
pub enum Color {
    Single(Option<HSBK>),
    Multi(Vec<Option<HSBK>>),
}

#[derive(Clone, Debug)]
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
    pub(crate) socket: UdpSocket,
    pub(crate) target: u64,
    pub(crate) name: RefreshableData<String>,
    pub(crate) power: RefreshableData<PowerLevel>,
    pub(crate) color: RefreshableData<Color>,
    pub(crate) model: RefreshableData<(u32, u32)>,
    pub(crate) location: RefreshableData<Location>,
    pub(crate) host_firmware: RefreshableData<u32>,
    pub(crate) wifi_firmware: RefreshableData<u32>,
}

impl Bulb {
    pub fn new(addr: SocketAddr, socket: UdpSocket, target: u64) -> Self {
        let short = Duration::from_secs(5);
        let long = Duration::from_secs(5 * 60); // 5 minutes
        Self {
            addr,
            socket,
            target,
            name: RefreshableData::with_config(long, Message::GetLabel),
            power: RefreshableData::with_config(short, Message::GetPower),
            color: RefreshableData::with_dyn_config(short, |color| match color {
                Some(Color::Single(_)) => Some(Message::LightGet),
                Some(Color::Multi(_)) => Some(Message::GetColorZones {
                    start_index: 0,
                    end_index: 255,
                }),
                None => Some(Message::GetVersion),
            }),
            model: RefreshableData::with_config(long, Message::GetVersion),
            location: RefreshableData::with_config(long, Message::GetLocation),
            host_firmware: RefreshableData::with_config(long, Message::GetHostFirmware),
            wifi_firmware: RefreshableData::with_config(long, Message::GetWifiFirmware),
        }
    }

    pub fn check(&mut self) -> Result<(), Error> {
        let build_opts = BuildOptions {
            target: Some(self.target),
            res_required: true,
            source: crate::client::CLIENT_IDENTIFIER,
            ..BuildOptions::default()
        };

        vec![
            self.name.check(),
            self.power.check(),
            self.color.check(),
            self.model.check(),
            self.location.check(),
            self.host_firmware.check(),
            self.wifi_firmware.check(),
        ]
        .into_iter()
        .filter_map(|msg| msg)
        .map(|msg| self.request_update(&build_opts, msg))
        .collect()
    }

    fn request_update(&self, build_opts: &BuildOptions, msg: Message) -> Result<(), Error> {
        let bytes = &RawMessage::build(build_opts, msg)?.pack()?;
        self.socket.send_to(bytes, self.addr)?;
        Ok(())
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn set_name(&self) {}

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
