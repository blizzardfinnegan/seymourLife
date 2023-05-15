use serialport;
use crate::serialport::SerialPortInfo;
use seymour_poc_rust::{device::Device, tty::{self,TTY,Response},log_facade,gpio_facade::GpioPins};
use std::io::{stdin,stdout,Write};
use std::thread::{self, JoinHandle};

fn int_input_filtering(prompt:Option<&str>) -> u64{
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
    return user_input.parse().unwrap_or(0);
}

fn input_filtering(prompt:Option<&str>) -> String{
    let internal_prompt = prompt.unwrap_or(">>>");
    let mut user_input:String = String::new();
    print!("{}",internal_prompt);
    _ = stdout().flush();
    stdin().read_line(&mut user_input).ok().expect("Did not enter a correct string");
    if let Some('\n')=user_input.chars().next_back() {
        user_input.pop();
    }
    if let Some('\r')=user_input.chars().next_back() {
        user_input.pop();
    }
    log::debug!("{}:{}",internal_prompt,user_input);
    return user_input;
}

fn main(){
    _ = log_facade::setup_logs();
    let gpio = &mut GpioPins::new();
    let available_ttys = std::fs::read_dir("/dev/serial/by-id").unwrap();
    let mut possible_devices:Vec<Option<Device>> = Vec::new();
    let mut tty_test_threads:Vec<JoinHandle<Option<Device>>> = Vec::new();
    for possible_tty in available_ttys.to_vec(){
        tty_test_threads.push(thread::spawn(move ||{
            let mut possible_port = TTY::new(possible_tty.as_ref().unwrap().path().to_str().as_ref());
            log::info!("Testing port {}. This may take a moment...",possible_tty.as_ref().unwrap().path().to_string_lossy());
            possible_port.write_to_device(tty::Command::Newline);
            let response = possible_port.read_from_device(Some(":"));
            if response != Response::Empty{
                log::debug!("{} is valid port!",possible_tty.as_ref().unwrap().path().to_string_lossy());
                Some(Device::new(possible_port,Some(response)))
            }
            else{
                None
            }
        }));
    }
    for thread in tty_test_threads{
        let output = thread.join().unwrap_or_else(|x|{log::trace!("{:?}",x); None});
        possible_devices.push(output);
    }

    let mut devices:Vec<Device> = Vec::new();
    for possible_device in possible_devices.into_iter(){
        if let Some(device) = possible_device{
            devices.push(device);
        }
    }

    log::info!("Number of devices detected: {}",devices.len());

    log::info!("Dimming all screens...");
    for device in devices.iter_mut(){
        device.darken_screen();
    }

    for device in devices.iter_mut(){
        device.brighten_screen()
            .set_serial(&input_filtering(Some("Enter the serial of the device with the bright screen: ")).to_string())
        .darken_screen();
        log::debug!("Number of unassigned addresses: {}",gpio.get_unassigned_addresses().len());
        for &address in gpio.get_unassigned_addresses(){
            device.set_pin_address(address).start_temp();
            if device.is_temp_running(){
                device.stop_temp();
                gpio.remove_address(address);
                break;
            }
            else{
                device.stop_temp();
            }
        }
    }

    let mut iteration_count:u64 = 0;
    while iteration_count < 1{
        iteration_count = int_input_filtering(Some("Enter the number of iterations to complete: "));
    }

    let mut iteration_threads = Vec::new();
    while let Some(mut device) = devices.pop(){
        iteration_threads.push(thread::spawn(move||{
            for i in 1..=iteration_count{
                log::info!("Starting iteration {} of {} for device {}...",
                               i,iteration_count,device.get_serial());
                device.test_cycle(None, None);
            }
        }));
    }
    for thread in iteration_threads{
        thread.join().unwrap();
    }
}
