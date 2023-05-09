pub mod gpio_facade{
    use rppal::gpio::{Gpio, OutputPin};
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::collections::hash_map::ValuesMut;

    const GPIO:Lazy<rppal::gpio::Gpio> = Lazy::new(|| Gpio::new().unwrap());
    const RELAY_ADDRESSES: [u8;10] = [4,5,6,12,13,17,18,19,20,26];

    pub struct GpioFacade{
        relays:HashMap<u8,OutputPin>
    }

    impl GpioFacade{
        pub fn new() -> Self {
            let mut output = Self { relays:HashMap::new() };
            for pin in RELAY_ADDRESSES.iter(){
                output.relays.entry(*pin).or_insert(GPIO.get(*pin).unwrap().into_output());
            }
            return output;
        }
        
        pub fn remove_pin(&mut self, address:u8) -> OutputPin{
            return self.relays.remove(&address).unwrap();
        }

        pub fn get_unassigned_pins(&mut self) -> ValuesMut<u8,OutputPin>{
            return self.relays.values_mut();
        }
    }
}
