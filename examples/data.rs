use lifx_lan;

fn main() -> Result<(), failure::Error> {
    let client = lifx_lan::start()?;
    std::thread::sleep(std::time::Duration::from_secs(1));

    loop {
        for bulb in client.bulbs.lock().unwrap().values_mut() {
            bulb.check()?;
        }
        for bulb in client.bulbs.lock().unwrap().values() {
            println!("{:?}: {:?} {:?}", bulb.name() , bulb.power(), bulb.color_single());
        }
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
}
