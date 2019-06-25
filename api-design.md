bulb = client.get_bulb_name("light")

bulb.name().keep_updated(Duration::from_secs(5))
bulb.name().on_update(|value| {})
bulb.name().get()
bulb.name().get_blocking()
bulb.name().refresh()
