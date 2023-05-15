const RELAY_ADDRESSES: [u8;10] = [4,5,6,12,13,17,18,19,20,26];

pub struct GpioPins{
    unassigned_addresses:Vec<u8>
}

impl GpioPins{
    pub fn new() -> Self {
        let mut output = Self { unassigned_addresses:Vec::new() };
        for pin in RELAY_ADDRESSES.iter(){
            let gpio_object = rppal::gpio::Gpio::new().unwrap();
            let pin_object = gpio_object.get(pin).unwrap().into_output();
            pin_object.set_low();
            output.unassigned_addresses.push(*pin);
        }
        return output;
    }

    pub fn remove_address(&mut self, address:u8) -> &mut Self {
        self.unassigned_addresses.retain(|x| *x != address);
        self
    }
    
    pub fn get_unassigned_addresses(&self) -> &Vec<u8>{
        return &self.unassigned_addresses;
    }
}
