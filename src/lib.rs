mod bulb;
mod client;
mod refreshable_data;

pub use bulb::Bulb;
use client::Client;

pub fn start() -> Result<Client, failure::Error> {
    let client = Client::new()?;
    client.discover_lights()?;
    Ok(client)
}
