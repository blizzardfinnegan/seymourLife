use rppal::gpio::{Gpio, OutputPin};
use once_cell::sync::Lazy;

const GPIO:Lazy<rppal::gpio::Gpio> = Lazy::new(|| Gpio::new().unwrap());
const RELAY_ADDRESSES: [u8;10] = [4,5,6,12,13,17,18,19,20,26];

pub struct GpioFacade{
    unassigned_relays:Vec<Box<Relay>>
}

pub struct Relay{
    relay:Box<OutputPin>
}

impl Relay{
    pub fn new(pin:OutputPin) -> Self{
        Self{ relay:Box::new(pin) }
    }
    pub fn low(&mut self) -> &mut Self{
        self.relay.set_low();
        return self;
    }
    pub fn high(&mut self) -> &mut Self{
        self.relay.set_high();
        return self;
    }
    pub fn address(&mut self) -> u8 {
        return self.relay.pin();
    }
}

impl GpioFacade{
    pub fn new() -> Self {
        let mut output = Self { unassigned_relays:Vec::new() };
        for pin in RELAY_ADDRESSES.iter(){
            output.unassigned_relays.push(Box::new(Relay::new(GPIO.get(*pin).unwrap().into_output())));
        }
        return output;
    }
    
    pub fn remove_pin(&mut self, address:u8) -> Option<&mut Box<Relay>>{
        for relay in self.unassigned_relays.iter_mut() {
            if &relay.address() == &address{

                return Some(relay);
            }
        }
        return None;
    }

    pub fn get_unassigned_relays(&mut self) -> &Vec<Box<Relay>>{
        return &self.unassigned_relays;
    }
}
