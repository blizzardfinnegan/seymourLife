use rppal::gpio::{Gpio, OutputPin};
use std::collections::HashMap;

const GPIO: rppal::gpio::Gpio = Gpio::new()?;
const RELAY_ADDRESSES: u8 = [4,5,6,12,13,17,18,19,20,26];
static mut RELAYS: HashMap<u8,OutputPin> = HashMap::new();

fn setup() {
    for pin in RELAY_ADDRESSES.iter(){
        gpio_facade::RELAYS.insert(pin,GPIO.get(pin)?.into_output());
    }
}

fn remove_pin(address:u8) -> OutputPin{
    return gpio_facade::RELAYS.remove(&address);
}