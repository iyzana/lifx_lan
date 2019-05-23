use failure::Error;
use lifx_core::{BuildOptions, Message, PowerLevel, RawMessage, HSBK};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

// random unique client identifier
const CLIENT_IDENTIFIER: u32 = 646_994_787;

type BoxUpdateFn<T> = Box<dyn FnMut(&UdpSocket, &BuildOptions, Option<&T>) -> Result<(), Error>>;

struct RefreshableData<T> {
    data: Option<T>,
    max_age: Duration,
    last_updated: Instant,
    queue_update: BoxUpdateFn<T>,
}

impl<T> RefreshableData<T> {
    fn with_config<F>(max_age: Duration, queue_update: F) -> Self
    where
        F: FnMut(&UdpSocket, &BuildOptions, Option<&T>) -> Result<(), Error> + 'static,
    {
        Self {
            data: None,
            max_age,
            last_updated: Instant::now(),
            queue_update: Box::new(queue_update),
        }
    }

    fn check(&mut self, socket: &UdpSocket, opts: &BuildOptions) -> Result<(), Error> {
        if self.data.is_none() || self.last_updated.elapsed() > self.max_age {
            (self.queue_update)(socket, opts, self.data.as_ref())?;
        }

        Ok(())
    }

    fn update(&mut self, data: T) {
        self.data = Some(data);
    }
}

pub enum Color {
    Single(HSBK),
    Multi(Vec<Option<HSBK>>),
}

pub struct Bulb {
    port: u32,
    target: u64,
    addr: SocketAddr,
    power_level: RefreshableData<PowerLevel>,
    color: RefreshableData<Color>,
}

impl Bulb {
    pub fn new(port: u32, target: u64, addr: SocketAddr) -> Self {
        let short = Duration::from_secs(15);
        Self {
            port,
            target,
            addr,
            power_level: RefreshableData::with_config(short, move |socket, opts, _| {
                Self::request_update(socket, opts, &addr, Message::GetPower)
            }),
            color: RefreshableData::with_config(short, move |socket, opts, color| match color {
                Some(Color::Single(_)) => {
                    Self::request_update(socket, opts, &addr, Message::LightGet)
                }
                Some(Color::Multi(_)) => Self::request_update(
                    socket,
                    opts,
                    &addr,
                    Message::GetColorZones {
                        start_index: 0,
                        end_index: 255,
                    },
                ),
                None => Ok(()),
            }),
        }
    }

    fn request_update(
        socket: &UdpSocket,
        opts: &BuildOptions,
        addr: &SocketAddr,
        msg: Message,
    ) -> Result<(), Error> {
        socket.send_to(&RawMessage::build(opts, msg)?.pack()?, addr)?;

        Ok(())
    }

    pub fn check(&mut self, socket: &UdpSocket) -> Result<(), Error> {
        let opts = BuildOptions {
            target: Some(self.target),
            res_required: true,
            source: CLIENT_IDENTIFIER,
            ..BuildOptions::default()
        };

        self.power_level.check(socket, &opts)?;
        self.color.check(socket, &opts)?;

        Ok(())
    }

    pub fn power_level(&self) -> Option<&PowerLevel> {
        self.power_level.data.as_ref()
    }

    pub fn color(&self) -> Option<&Color> {
        self.color.data.as_ref()
    }
}
