use rppal::gpio::Gpio;

const RELAY_ADDRESSES: [u8;10] = [4,5,6,12,13,17,18,19,20,26];

pub struct GpioPins{
    unassigned_addresses:Vec<u8>
}

impl GpioPins{
    pub fn new() -> Self {
        let mut output = Self { unassigned_addresses:Vec::new() };
        for pin in RELAY_ADDRESSES.iter(){
            match Gpio::new(){
                Ok(gpio_object) =>{
                    match gpio_object.get(pin.clone()){
                        Ok(pin_object)=>{
                            _ = pin_object.into_output_low();
                            output.unassigned_addresses.push(*pin);
                        },
                        Err(error) => {
                            log::warn!("Pin unavailable!");
                            log::debug!("{}",error);
                        }
                    }
                },
                Err(error) => {
                    log::warn!("Unable to open GPIO!");
                    log::debug!("{}",error);
                }
            }
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
