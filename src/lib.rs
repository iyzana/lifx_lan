mod bulb;

pub use bulb::Bulb;

pub fn start() -> Bulb {
    Bulb::new(0, 0, "[::1]:80".parse().unwrap())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
