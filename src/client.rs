use crate::bulb::{self, Bulb};
use chrono::prelude::*;
use get_if_addrs::*;
use lifx_core::{get_product_info, BuildOptions, LifxIdent, LifxString, Message, RawMessage};
use log::warn;
use std::{
    collections::HashMap,
    net::UdpSocket,
    sync::{Arc, Mutex},
    thread,
};

type BulbMap = Arc<Mutex<HashMap<u64, Bulb>>>;

pub struct Client {
    pub bulbs: BulbMap,
    pub socket: UdpSocket,
}

impl Client {
    pub fn new() -> Result<Self, failure::Error> {
        let socket = UdpSocket::bind("0.0.0.0:56700")?;
        let bulbs = Arc::new(Mutex::new(HashMap::new()));

        socket.set_broadcast(true)?;

        {
            let socket = socket.try_clone()?;
            let bulbs = Arc::clone(&bulbs);
            Self::handle_responses(socket, bulbs);
        }

        Ok(Self { bulbs, socket })
    }

    pub fn discover_lights(&self) -> Result<(), failure::Error> {
        let opts = BuildOptions::default();
        let msg = RawMessage::build(&opts, Message::GetService)
            .unwrap()
            .pack()
            .unwrap();

        get_if_addrs()
            .expect("could not list interfaces")
            .into_iter()
            .filter_map(|interface| match interface.addr {
                IfAddr::V4(if_addr) => Some(if_addr),
                _ => None,
            })
            .filter(|if_addr| !if_addr.ip.is_loopback())
            .filter_map(|if_addr| if_addr.broadcast)
            .map(|broadcast| {
                self.socket.send_to(&msg, (broadcast, 56700))?;
                Ok(())
            })
            .collect::<Result<(), failure::Error>>()
    }

    fn handle_responses(socket: UdpSocket, bulbs: BulbMap) {
        thread::spawn(move || {
            let mut buf = [0; 1024];
            loop {
                let (num_bytes, addr) = socket
                    .recv_from(&mut buf)
                    .expect("failed to read from socket");
                match RawMessage::unpack(&buf[..num_bytes]) {
                    Ok(ref raw) if raw.frame_addr.target == 0 => continue,
                    Ok(raw) => {
                        let mut bulbs = bulbs.lock().unwrap();
                        let bulb = bulbs
                            .entry(raw.frame_addr.target)
                            .and_modify(|bulb: &mut Bulb| bulb.addr = addr)
                            .or_insert_with(|| Bulb::new(addr, raw.frame_addr.target));

                        match Message::from_raw(&raw) {
                            Ok(msg) => Self::handle_message(bulb, msg),
                            Err(err) => warn!("Could not decode message: {}", err),
                        }
                    }
                    Err(err) => warn!("Could not parse message: {}", err),
                }
            }
        });
    }

    fn handle_message(bulb: &mut Bulb, msg: Message) {
        match msg {
            Message::StateLabel { label } => bulb.name.update(label.0),
            Message::StateLocation {
                location: LifxIdent(id),
                label: LifxString(label),
                updated_at,
            } => bulb.location.update(bulb::Location::new(
                id,
                label,
                Utc.timestamp(
                    (updated_at / 1_000_000_000) as i64,
                    (updated_at % 1_000_000_000) as u32,
                ),
            )),
            Message::StatePower { level } => bulb.power.update(level),
            Message::StateVersion {
                vendor, product, ..
            } => {
                bulb.model.update((vendor, product));
                if let Some(info) = get_product_info(vendor, product) {
                    bulb.color.update(if info.multizone {
                        bulb::Color::Multi(vec![])
                    } else {
                        bulb::Color::Single(None)
                    });
                }
            }
            Message::LightState {
                color,
                power,
                label,
                ..
            } => {
                bulb.name.update(label.0);
                if let Some(bulb::Color::Single(bulb_color)) = bulb.color.as_mut() {
                    *bulb_color = Some(color);
                    bulb.power.update(power);
                }
            }
            _ => {}
        }
    }
}
