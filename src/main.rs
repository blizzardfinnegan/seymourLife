use seymour_poc_rust::{device::Device, 
                       tty::{self,TTY,Response},
                       gpio_facade::GpioPins};
use std::{io::{stdin,stdout,Write},
          thread::{self, JoinHandle},
          path::Path,
          fs};
use chrono::{DateTime,Local};
use clap::Parser;

#[derive(Parser,Debug)]
#[command(author,version,about)]
struct Args{
    #[arg(short,long,action)]
    debug:bool
}

const VERSION:&str="2.2.0";

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
    let args = Args::parse();
    setup_logs(&args.debug);
    log::info!("Seymour Life Testing version: {}",VERSION);
    if args.debug{
        log::debug!("Debug enabled!");
    }
    loop{
        let gpio = &mut GpioPins::new();
        match std::fs::read_dir("/dev/serial/by-path"){
            Ok(available_ttys)=>{
                let mut possible_devices:Vec<Option<Device>> = Vec::new();
                let mut tty_test_threads:Vec<JoinHandle<Option<Device>>> = Vec::new();
                for possible_tty in available_ttys.into_iter(){
                    tty_test_threads.push(
                        thread::spawn(move ||{
                            let tty_ref = possible_tty.as_ref();
                            match tty_ref{
                                Ok(tty_real_ref)=>{
                                    let tty_path =  tty_real_ref.path();
                                    let tty_name = tty_path.to_string_lossy();
                                    log::info!("Testing port {}. This may take a moment...",&tty_name);
                                    let possible_port = TTY::new(&tty_name);
                                    match possible_port{
                                        Some(mut port) =>{
                                            port.write_to_device(tty::Command::Newline);
                                            let response = port.read_from_device(Some(":"));
                                            if response != Response::Empty{
                                                log::debug!("{} is valid port!",tty_name);
                                                let new_device = Device::new(port,Some(response));
                                                match new_device{
                                                    Ok(mut device) => {
                                                        device.darken_screen();
                                                        Some(device)
                                                    },
                                                    Err(_) => None
                                                }
                                            }
                                            else { None }
                                        },
                                        None=>{None}
                                    }
                                },
                                Err(error)=>{
                                    log::debug!("{}",error);
                                    None
                                }
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

                log::info!("--------------------------------------");
                log::info!("Number of devices detected: {}",devices.len());
                log::info!("--------------------------------------\n\n");

                for device in devices.iter_mut(){
                    device.brighten_screen();
                    if args.debug{
                        let location = device.get_location();
                        log::info!("Init device {}...", location);
                        device.set_serial(&location);
                    }
                    else{
                        device.set_serial(&input_filtering(Some("Enter the serial of the device with the bright screen: ")).to_string());
                    }
                    device.darken_screen();
                    log::debug!("Number of unassigned addresses: {}",gpio.get_unassigned_addresses().len());
                    if !find_gpio(device, gpio){
                        device.set_pin_address(21);
                        log::error!("Unable to find GPIO for device {}. Please ensure that the probe well is installed properly, and the calibration key is plugged in.",device.get_location());
                    }
                }

                let mut iteration_count:u64 = 0;
                if args.debug { 
                    iteration_count = 10000;
                }
                else {
                    while iteration_count < 1{
                        iteration_count = int_input_filtering(Some("Enter the number of iterations to complete: "));
                    }
                }

                let mut iteration_threads = Vec::new();
                while let Some(mut device) = devices.pop(){
                    iteration_threads.push(thread::spawn(move||{
                        device.init_temp_count();
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
            Err(_)=>{
                log::error!("Invalid serial location! Please make sure that /dev/serial/by-path exists.");
                break;
            }
        }
        if input_filtering(Some("Would you like to run the tests again? (y/N): ")).to_string().contains("y") {}
        else { break; }
    }
}

fn find_gpio(device:&mut Device,gpio:&mut GpioPins) -> bool{
    device.init_temp_count();
    for &address in gpio.get_unassigned_addresses(){
        device.set_pin_address(address).start_temp();
        if device.is_temp_running(){
            device.stop_temp();
            gpio.remove_address(address);
            return true;
        }
        else {
            device.stop_temp();
        }
    }
    return false;
}

pub fn setup_logs(debug:&bool){
    let chrono_now: DateTime<Local> = Local::now();
    if ! Path::new("logs").is_dir(){
        _ = fs::create_dir("logs");
    };
    _ = fern::Dispatch::new()
        .format(|out,message,record|{
            out.finish(format_args!(
                "{} - [{}, {}] - {}",
                Local::now().to_rfc3339(),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(
            fern::Dispatch::new()
                .level(log::LevelFilter::Trace)
                .chain(fern::log_file(
                    format!("logs/{0}.log",
                    chrono_now.format("%Y-%m-%d_%H.%M").to_string()
                    )).unwrap()),
        )
        .chain({
            let mut stdout_logger = fern::Dispatch::new();
            if *debug {
                stdout_logger = stdout_logger.level(log::LevelFilter::Trace);
            }
            else {
                stdout_logger = stdout_logger.level(log::LevelFilter::Info);
            }
                stdout_logger.chain(std::io::stdout())
        })
        .apply();
}
