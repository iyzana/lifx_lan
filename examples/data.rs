use lifx_lan;

fn main() -> Result<(), failure::Error> {
    let client = lifx_lan::start()?;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        for bulb in client.bulbs.lock().unwrap().values_mut() {
            bulb.check(&client.socket)?;
        }
        for bulb in client.bulbs.lock().unwrap().values() {
            println!("{:?}: {:?} {:?}", bulb.name() , bulb.power(), bulb.color_single());
        }
    }
}
