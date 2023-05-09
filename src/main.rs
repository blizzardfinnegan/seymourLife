#[allow(unused_imports)]
use log::{warn,error,debug,info,trace};
use serialport;
use crate::serialport::SerialPortInfo;
use seymour_poc_rust::{device, tty,log_facade,gpio_facade::{Relay, self, GpioFacade}, tty::TTY};
use std::io::{stdin,stdout,Write};

#[allow(unused_imports)]
use std::{thread, time::Duration};

fn int_input_filtering(prompt:Option<&str>) -> i64{
    let internal_prompt = prompt.unwrap_or(">>>");
    let mut user_input:String = String::new();
    print!("{}",internal_prompt);
    _ = stdout().flush();
    stdin().read_line(&mut user_input).expect("Did not input a valid number.");
    if let Some('\n')=user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r')=user_input.chars().next_back() {
        user_input.pop();
    }
    return user_input.parse().unwrap_or(-1);
}

fn input_filtering(prompt:Option<&str>) -> String{
    let internal_prompt = prompt.unwrap_or(">>>");
    let mut user_input:String = String::new();
    print!("{}",internal_prompt);
    _ = stdout().flush();
    stdin().read_line(&mut user_input).expect("Did not enter a correct string");
    if let Some('\n')=user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r')=user_input.chars().next_back() {
        user_input.pop();
    }
    return user_input;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_facade::setup_logs().unwrap();
    let mut gpio = GpioFacade::new();
    let available_ttys:Vec<SerialPortInfo> = tty::AVAILABLE_TTYS.clone();
    let mut possible_devices:Vec<Option<device::Device>> = Vec::new();
    for possible_tty in available_ttys.to_vec(){
        possible_devices.push(thread::spawn(move ||{
            let mut possible_port = TTY::new(possible_tty.port_name.to_string());
            println!("Testing port {}. This may take a moment...",possible_tty.port_name);
            possible_port.write_to_device(tty::Command::Newline);
            let response = possible_port.read_from_device(Some(""));
            if response != tty::Response::Other{
                Some(device::Device::new(possible_port))
            }
            else{
                None
            }
        }).join().unwrap());
    }
    let mut devices:Vec<device::Device> = Vec::new();
    for possible_device in possible_devices.into_iter(){
        if let Some(device) = possible_device{
            devices.push(device);
        }
    }

    println!("Dimming all screens...");
    for device in devices.iter_mut(){
        device.darken_screen();
    }

    for device in devices.iter_mut(){
        device.brighten_screen()
            .set_serial(&input_filtering(Some("Enter the serial of the device with the bright screen: ")).to_string())
        .darken_screen();
        let unassigned_relays:&Vec<Box<Relay>> = gpio.get_unassigned_relays();
        for relay in unassigned_relays.into_iter(){
            _ = &relay.high();
            if device.is_temp_running(){
                device.set_gpio(**relay);
            }
        }
    }

    Ok(())
}
