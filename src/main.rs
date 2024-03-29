use seymour_life::{device::Device, 
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
    /// Print all logs to screen, improves log verbosity. Sets iteration count to 50000
    #[arg(short,long,action)]
    debug:bool,

    /// Force manually setting serial numbers
    #[arg(short,long,action)]
    manual:bool,

    /// Set iteration count from command line. Overrides debug iteration count.
    #[arg(short,long)]
    iterations:Option<u64>

}

const VERSION:&str="2.3.3";
const DEBUG_ITERATION_COUNT:u64=50000;

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
//Path::new(&&str).is_dir() -> bool
fn main(){
    let args = Args::parse();
    setup_logs(&args.debug);
    log::info!("Seymour Life Testing version: {}",VERSION);
    log::trace!("Debug enabled!");
    loop{
        let mut iteration_count:u64 = 0;
        if let Some(value) = args.iterations{
            iteration_count = value;
        }
        else if args.debug { 
            iteration_count = DEBUG_ITERATION_COUNT;
        }
        else {
            while iteration_count < 1{
                iteration_count = int_input_filtering(Some("Enter the number of iterations to complete: "));
            }
        }

        log::info!("Testing all available USB ports for connected devices. This may take several minutes, and devices may reboot several times.");
        let gpio = &mut GpioPins::new();
        let mut available_ttys:Vec<Box<Path>> = Vec::new();
        for entry in glob::glob("/dev/serial/*").expect("Failed to read glob pattern"){
            match entry{
                Ok(real_path) =>{
                    match fs::read_dir::<&Path>(real_path.as_ref()){
                        Ok(possible_ttys) =>{
                            possible_ttys.into_iter().for_each(|tty| {
                                if let Ok(single_tty) = tty {
                                    available_ttys.push(single_tty.path().into());
                                }
                            });
                            break;
                        }
                        Err(error) =>{
                            log::error!("Invalid permissions to /dev directory... did you run with sudo?");
                            log::error!("{}",error);
                            return;
                        }
                    }
                }
                Err(error) =>{
                    log::error!("{}",error);
                }
            }
        }
        if available_ttys.is_empty(){
            for entry in glob::glob("/dev/ttyUSB*").expect("Unable to read glob"){
                match entry{
                    Ok(possible_tty) => available_ttys.push(Path::new(&possible_tty).into()),
                    Err(error) => {
                        log::error!("Invalid permissions to /dev directory... did you run with sudo?");
                        log::error!("{}",error);
                        return;
                    }
                };
            }
        }

        if available_ttys.is_empty(){
            log::error!("No serial devices detected! Please ensure all connections.");
            return;
        }
        let mut possible_devices:Vec<Option<Device>> = Vec::new();
        let mut tty_test_threads:Vec<JoinHandle<Option<Device>>> = Vec::new();
        for possible_tty in available_ttys.into_iter(){
            tty_test_threads.push(
                thread::spawn(move ||{
                    let tty_name = possible_tty.to_string_lossy();
                    log::debug!("Testing port {}",&tty_name);
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
                                        if !args.manual {
                                            device.auto_set_serial();
                                        }
                                        Some(device)
                                    },
                                    Err(_) => None
                                }
                            }
                            else { None }
                        },
                        None=>{None}
                    }
            }));
        }
        for thread in tty_test_threads{
            let output = thread.join().unwrap_or_else(|x|{log::trace!("{:?}",x); None});
            possible_devices.push(output);
        }

        let mut serials_set:bool = true;
        let mut devices:Vec<Device> = Vec::new();
        for possible_device in possible_devices.into_iter(){
            if let Some(device) = possible_device{
                if device.get_serial().eq("uninitialised"){
                    serials_set = false;
                }
                devices.push(device);
            }
        }

        log::info!("--------------------------------------");
        log::info!("Number of devices detected: {}",devices.len());
        log::info!("--------------------------------------\n\n");

        log::info!("Setting up probe wells for all devices. This may take several minutes...");
        for device in devices.iter_mut(){
            if !serials_set || args.manual {
            device.brighten_screen();
            device.manual_set_serial(&input_filtering(Some("Enter the serial of the device with the bright screen: ")).to_string());
            device.darken_screen();
            }
            log::info!("Checking probe well of device {}",device.get_serial());
            log::debug!("Number of unassigned addresses: {}",gpio.get_unassigned_addresses().len());
            if !find_gpio(device, gpio){
                device.set_pin_address(21);
                log::error!("Unable to find probe-well for device {}. Please ensure that the probe well is installed properly, and the calibration key is plugged in.",device.get_serial());
                device.brighten_screen();
                panic!();
            }
        }

        let mut iteration_threads = Vec::new();
        while let Some(mut device) = devices.pop(){
            iteration_threads.push(thread::spawn(move||{
                device.init_temp_count();
                for i in 1..=iteration_count{
                    log::info!("Starting iteration {} of {} for device {}...",
                                   i,iteration_count,device.get_serial());
                    device.test_cycle(None);
                }
            }));
        }
        for thread in iteration_threads{
            thread.join().unwrap();
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
        .chain({
            let mut file_logger = fern::Dispatch::new();
            let date_format = chrono_now.format("%Y-%m-%d_%H.%M").to_string();
            let local_log_file = fern::log_file(format!("logs/{}.log",date_format)).unwrap();
            if *debug{
                file_logger = file_logger.level(log::LevelFilter::Trace);
            }
            else {
                file_logger = file_logger.level(log::LevelFilter::Debug);
            }
            file_logger.chain(local_log_file)
        })
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
