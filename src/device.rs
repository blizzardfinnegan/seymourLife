use std::{fs::{self, File}, path::Path, io::Write, thread, time::Duration};
use crate::tty::{TTY, Response,Command};
use rppal::gpio::{Gpio,OutputPin};

const TEMP_WAIT:Duration = Duration::from_secs(3);
const REBOOTS_SECTION: &str = "Reboots: ";
const BP_SECTION: &str = "Successful BP tests: ";
const TEMP_SECTION: &str = "Successful temp tests: ";
const OUTPUT_FOLDER: &str = "output/";
const UNINITIALISED_SERIAL: &str = "uninitialised";
#[derive(PartialEq,Debug)]
pub enum State{
    Shutdown,
    LoginPrompt,
    DebugMenu,
    LifecycleMenu,
    BrightnessMenu
}

#[derive(Debug)]
pub struct Device{
    usb_tty:TTY,
    output_file: Option<File>,
    gpio: rppal::gpio::Gpio,
    address: Option<u8>,
    pin: Option<OutputPin>,
    serial: String,
    current_state: State,
    reboots: u64,
    temps: u64,
    init_temps: u64,
    bps: u64
}

impl Device{
    fn load_values(&mut self) -> bool {
        if ! Path::new(&OUTPUT_FOLDER).is_dir(){
            _ = fs::create_dir(&OUTPUT_FOLDER);
        };
        log::debug!("{:?}",&self.serial);
        let output_path:String = OUTPUT_FOLDER.to_owned() + &self.serial + ".txt";
        if ! Path::new(&output_path).exists(){
            log::debug!("Creating file {}",&output_path);
            let temp:Result<File, std::io::Error> = fs::File::create(&output_path);
            match temp{
                Ok(file) => {
                    self.output_file = Some(file);
                    self.save_values();
                }
                Err(_) => {
                    return false
                }
            }
        }
        else {
            let temp:Result<String, std::io::Error> = std::fs::read_to_string(output_path);
            match temp{
                Ok(file_contents) =>{
                    let file_lines:Vec<&str> = file_contents.split("\n").collect();
                    log::trace!("{:?}",file_contents);
                    for line in file_lines {
                        if line.len() > 0{
                            log::trace!("{:?}",line);
                            let section_and_data:Vec<&str> = line.split(": ").collect();
                            let section:&str = section_and_data[0];
                            let possible_value:Result<u64, std::num::ParseIntError> = section_and_data[1].trim().parse::<u64>();
                            match possible_value{
                                Ok(value) => {
                                    log::trace!("{:?} value: [{:?}]",section,value);
                                    match section {
                                        REBOOTS_SECTION => {
                                            self.reboots = value;
                                        },
                                        BP_SECTION => {
                                            self.bps = value;
                                        },
                                        TEMP_SECTION => {
                                            self.temps = value;
                                        },
                                        _ => ()
                                    };
                                }
                                Err(_) => {
                                    log::warn!("Unable to parse value [{:?}] into integer",section_and_data);
                                }
                            }
                        };
                    };
                },
                Err(error) => {
                    log::warn!("Could not load from file!");
                    log::debug!("{}",error);
                }
            }
        };
        return true
    }
    pub fn new(mut usb_port:TTY,response:Option<Response>) -> Result<Self,String>{
        let initial_state:State;
        match response{
            Some(response_value)=> {
                match response_value{
                    Response::PasswordPrompt=>{
                        usb_port.write_to_device(Command::Newline);
                        _ = usb_port.read_from_device(None);
                        initial_state = State::LoginPrompt;
                    },
                        //Response::Empty parsing here is potentially in bad faith
                    Response::Other | Response::Empty | Response::ShellPrompt | Response::FailedDebugMenu |
                    Response::LoginPrompt | Response::ShuttingDown | Response::Rebooting | Response::PreShellPrompt => 
                        initial_state = State::LoginPrompt,
                    Response::BPOn | Response::BPOff | Response::TempCount(_) |
                    Response::DebugMenu=>{
                        usb_port.write_to_device(Command::Quit);
                        _ = usb_port.read_from_device(None);
                        usb_port.write_to_device(Command::Newline);
                        match usb_port.read_from_device(None){
                            Response::Rebooting => {
                                while usb_port.read_from_device(None) != Response::LoginPrompt {}
                                initial_state = State::LoginPrompt;
                            },
                            Response::ShellPrompt => {
                                usb_port.write_to_device(Command::Shutdown);
                                while usb_port.read_from_device(None) != Response::LoginPrompt {}
                                initial_state = State::LoginPrompt;
                            },
                            _ => {
                                log::error!("Unknown state for TTY {:?}!!! Consult logs immediately.",usb_port);
                                return Err("Failed TTY init. Unknown state, cannot trust.".to_string());
                            }
                        };
                    },
                };
            },
            None => initial_state = State::LoginPrompt
        };
        let temp = Gpio::new();
        match temp{
            Ok(gpio) =>{
                let mut output = Self{
                    usb_tty: usb_port,
                    gpio,
                    address: None,
                    pin: None,
                    output_file: None,
                    serial: UNINITIALISED_SERIAL.to_string(),
                    current_state: initial_state,
                    reboots: 0,
                    temps: 0,
                    init_temps: 0,
                    bps: 0
                };
                if !output.load_values(){
                    log::warn!("Could not load values from file! File may be overwritten.");
                }
                return Ok(output);
            }
            Err(error) => {
                log::warn!("Failed to init GPIO!");
                log::debug!("{}",error);
                return Err("Failed GPIO init".to_string());
            }
        }
    }

    fn go_to_brightness_menu(&mut self) -> &mut Self{
        while !(self.current_state == State::BrightnessMenu){
            match self.current_state {
                State::BrightnessMenu => return self,
                State::DebugMenu => {
                    self.usb_tty.write_to_device(Command::LifecycleMenu);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::LifecycleMenu;
                },
                State::LifecycleMenu =>{
                    self.usb_tty.write_to_device(Command::BrightnessMenu);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::BrightnessMenu;
                    return self;
                },
                State::LoginPrompt => {
                    self.usb_tty.write_to_device(Command::Login);
                    loop {
                        match self.usb_tty.read_from_device(None){
                            Response::PreShellPrompt | Response::Empty | Response::ShuttingDown | Response::Rebooting => {},
                            Response::PasswordPrompt => {self.usb_tty.write_to_device(Command::Newline);},
                            Response::ShellPrompt => break,
                            _ => {
                                log::error!("Unexpected response from device {}!",self.serial);
                                log::error!("Unsure how to continue. Expect data from device {} to be erratic until next cycle.",self.serial);
                                break;
                            },
                        };
                    };
                    //_ = self.usb_tty.read_from_device(None);
                    self.usb_tty.write_to_device(Command::DebugMenu);
                    loop {
                        match self.usb_tty.read_from_device(None)   {
                            Response::PreShellPrompt | Response::Empty | Response::ShuttingDown | Response::Rebooting => {},
                            Response::LoginPrompt => {
                                self.usb_tty.write_to_device(Command::Login);
                                while self.usb_tty.read_from_device(None) != Response::ShellPrompt {};
                                self.usb_tty.write_to_device(Command::DebugMenu);
                            },
                            Response::DebugMenu =>
                                break,
                            Response::FailedDebugMenu => {
                                while self.usb_tty.read_from_device(None) != Response::LoginPrompt {};
                                self.usb_tty.write_to_device(Command::Login);
                                while self.usb_tty.read_from_device(None) != Response::ShellPrompt {};
                                self.usb_tty.write_to_device(Command::DebugMenu);
                            },
                            _ => { 
                                log::error!("Unexpected response from device {}!", self.serial);
                                log::error!("Unsure how to continue. Expect data from device {} to be erratic until next cycle.",self.serial);
                                break;
                            },
                        };
                    };
                    //_ = self.usb_tty.read_from_device(None);
                    self.current_state = State::DebugMenu;
                },
                State::Shutdown => {
                    while self.usb_tty.read_from_device(None) != Response::LoginPrompt{}
                    self.current_state = State::LoginPrompt;
                },
            };
        };
        return self;
    }

    fn go_to_lifecycle_menu(&mut self) -> &mut Self{
        while !(self.current_state == State::LifecycleMenu){
            match self.current_state {
                State::LifecycleMenu => return self,
                State::DebugMenu => {
                    self.usb_tty.write_to_device(Command::LifecycleMenu);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::LifecycleMenu;
                    return self;
                },
                State::BrightnessMenu =>{
                    self.usb_tty.write_to_device(Command::UpMenuLevel);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::LifecycleMenu;
                    return self;
                },
                State::LoginPrompt => {
                    self.usb_tty.write_to_device(Command::Login);
                    loop {
                        match self.usb_tty.read_from_device(None){
                            Response::PreShellPrompt | Response::Empty | Response::ShuttingDown | Response::Rebooting => {},
                            Response::PasswordPrompt => {self.usb_tty.write_to_device(Command::Newline);},
                            Response::ShellPrompt => break,
                            _ => {
                                log::error!("Unexpected response from device {}!",self.serial);
                                log::error!("Unsure how to continue. Expect data from device {} to be erratic until next cycle.",self.serial);
                                break;
                            },
                        };
                    };
                    self.usb_tty.write_to_device(Command::DebugMenu);
                    loop {
                        match self.usb_tty.read_from_device(None)   {
                            Response::PreShellPrompt | Response::Empty | Response::ShuttingDown | Response::Rebooting => {},
                            Response::LoginPrompt => {
                                self.usb_tty.write_to_device(Command::Login);
                                while self.usb_tty.read_from_device(None) != Response::ShellPrompt {};
                                self.usb_tty.write_to_device(Command::DebugMenu);
                            },
                            Response::DebugMenu =>
                                break,
                            Response::FailedDebugMenu => {
                                while self.usb_tty.read_from_device(None) != Response::LoginPrompt {};
                                self.usb_tty.write_to_device(Command::Login);
                                while self.usb_tty.read_from_device(None) != Response::ShellPrompt {};
                                self.usb_tty.write_to_device(Command::DebugMenu);
                            },
                            _ => { 
                                log::error!("Unexpected response from device {}!", self.serial);
                                log::error!("Unsure how to continue. Expect data from device {} to be erratic until next cycle.",self.serial);
                                break;
                            },
                        };
                    };
                    self.current_state = State::DebugMenu;
                },
                State::Shutdown => {
                    while self.usb_tty.read_from_device(None) != Response::LoginPrompt {}
                    self.current_state = State::LoginPrompt;
                },
            };
        };
        return self;
    }

    fn save_values(&mut self) -> bool{
        let output_path = OUTPUT_FOLDER.to_owned() + &self.serial + ".txt";
        let temp = fs::OpenOptions::new().write(true).truncate(true).open(&output_path);
        match temp{
            Ok(opened_file) => self.output_file = Some(opened_file),
            Err(_) => {
                log::warn!("Could not open file [{}] to write! Potential permissions error.",&output_path);
                return false
            }
        }
        log::trace!("Writing to file: {:?}",self.output_file);
        if let Some(ref mut file_name) = self.output_file{
            log::debug!("Writing to file!");
            let mut output_data = REBOOTS_SECTION.to_string();
            output_data.push_str(&self.reboots.to_string());
            output_data.push_str("\n");
            output_data.push_str(BP_SECTION);
            output_data.push_str(&self.bps.to_string());
            output_data.push_str("\n");
            output_data.push_str(TEMP_SECTION);
            log::trace!("Current temps: [{}]",self.temps);
            log::trace!("Initial temps: [{}]",self.init_temps);
            let saved_temps = self.temps - self.init_temps;
            output_data.push_str(&saved_temps.to_string());
            output_data.push_str("\n");
            let temp = file_name.write_all(output_data.as_bytes());
            match temp{
                Err(error) => {
                    log::warn!("{}",error);
                },
                _ => {}
            }
        }
        else {
            log::warn!("Cannot write to output file!");
        }
        return true
    }
    pub fn set_serial(&mut self, serial:&str) -> &mut Self{
        self.serial = serial.to_string();
        self.load_values();
        self.save_values();
        return self;
    }
    pub fn get_serial(&mut self) -> &str{
        &self.serial
    }
    pub fn get_location(&mut self) -> String{
        std::format!("{:?}",self.usb_tty)
    }
    pub fn set_pin_address(&mut self, address:u8) -> &mut Self{
        self.address = Some(address.clone());
        let temp = self.gpio.get(address);
        match temp{
            Ok(pin) => self.pin = Some(pin.into_output()),
            Err(error) => {
                log::warn!("Could not set pin to this address {}; already assigned?",address);
                log::debug!("{}",error);
            }
        }
        return self;
    }
    pub fn start_temp(&mut self) -> &mut Self {
        if let Some(ref mut pin) = self.pin {
            pin.set_high();
        }
        return self;
    }
    pub fn stop_temp(&mut self) -> &mut Self {
        if let Some(ref mut pin) = self.pin {
            pin.set_low();
        }
        return self;
    }
    fn start_bp(&mut self) -> &mut Self {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(Command::StartBP);
        _ = self.usb_tty.read_from_device(None);
        return self;
    }
    pub fn darken_screen(&mut self) -> &mut Self {
        self.go_to_brightness_menu();
        self.usb_tty.write_to_device(Command::BrightnessLow);
        _ = self.usb_tty.read_from_device(None);
        return self;
    }
    pub fn brighten_screen(&mut self) -> &mut Self {
        self.go_to_brightness_menu();
        self.usb_tty.write_to_device(Command::BrightnessHigh);
        _ = self.usb_tty.read_from_device(None);
        return self;
    }

    pub fn is_temp_running(&mut self) -> bool{
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(Command::ReadTemp);
        for _ in 0..10 {
            match self.usb_tty.read_from_device(None){
                Response::TempCount(count) => return count != self.init_temps ,
                _ => {},
            }
        }
	self.usb_tty.write_to_device(Command::ReadTemp);
	for _ in 0..10{
	    match self.usb_tty.read_from_device(None){
                Response::TempCount(count) => return count != self.init_temps ,
		_ => {},
	    }
        }
	log::error!("Temp read failed!!!");
        return false
    }

    pub fn update_temp_count(&mut self) -> u64 {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(Command::ReadTemp);
        for _ in 0..10 {
            match self.usb_tty.read_from_device(None){
                Response::TempCount(count) => {
                    log::trace!("Count for device {} updated to {}",self.serial,count);
                    self.temps = count;
                    return count
                },
                _ => {},
            }
        }
	self.usb_tty.write_to_device(Command::ReadTemp);
	for _ in 0..10{
	    match self.usb_tty.read_from_device(None){
                Response::TempCount(count) => {
                    log::trace!("Count for device {} updated to {}",self.serial,count);
                    self.temps = count;
                    return count
                },
		_ => {},
	    }
        }
	log::error!("Update temp count on device {} failed!!!",self.serial);
	return 0;
    }

    pub fn init_temp_count(&mut self){
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(Command::ReadTemp);
        for _ in 0..10 {
            match self.usb_tty.read_from_device(None){
                Response::TempCount(count) => {
                    log::trace!("init temp count set to {} on device {}",count,self.serial);
                    self.init_temps = count;
                    return
                },
                _ => {},
            }
        };
	self.usb_tty.write_to_device(Command::ReadTemp);
	for _ in 0..10{
	    match self.usb_tty.read_from_device(None){
                Response::TempCount(count) => {
                    log::trace!("init temp count set to {} on device {}",count,self.serial);
                    self.init_temps = count;
                    return
                },
		_ => {},
	    }
        };
	log::error!("init temp count failed on device {}!!!",self.serial);
    }

    fn is_bp_running(&mut self) -> bool {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(Command::CheckBPState);
        loop { 
            match self.usb_tty.read_from_device(None){
                Response::BPOn => return true,
                Response::BPOff => return false,
                _ => return false,
            }
        }
    }
    pub fn reboot(&mut self) -> () {
        self.usb_tty.write_to_device(Command::Quit);
        let mut successful_reboot:bool = false;
        let mut exited_menu:bool = false;
        loop{
            match self.usb_tty.read_from_device(None){
                Response::LoginPrompt => break,
                Response::Rebooting => {
                    log::trace!("Successful reboot detected for device {}.",self.serial);
                    successful_reboot = true;
                    if !exited_menu { log::info!("Unusual reboot detected for device {}. Please check logs.",self.serial); }
                },
                Response::ShuttingDown => {
                    log::trace!("Exiting debug menu on device {}.",self.serial);
                    exited_menu = true;
                },
                _ => {}
            }
        };
        if successful_reboot { self.reboots += 1; }
        self.current_state = State::LoginPrompt;
    }

    pub fn test_cycle(&mut self, bp_cycles: Option<u64>) -> () {
        let local_bp_cycles: u64 = bp_cycles.unwrap_or(3);
        if self.current_state != State::LoginPrompt { self.reboot(); }
        self.go_to_lifecycle_menu();
        _ = self.usb_tty.read_from_device(Some("["));
        self.update_temp_count();
        for _bp_count in 1..=local_bp_cycles{
            log::info!("Running bp {} on device {} ...",(self.bps+1),self.serial);
            self.start_bp();
            let bp_start:bool = self.is_bp_running();
            log::trace!("Has bp started on device {}? : {:?}",self.serial,bp_start);

            if bp_start{
                log::trace!("Starting temp on device {}",self.serial);
                self.start_temp();
                thread::sleep(TEMP_WAIT);
                log::trace!("Stopping temp on device {}",self.serial);
                self.stop_temp();
            };

            while self.is_bp_running() {};

            let bp_end = self.is_bp_running();
            log::trace!("Has bp ended on device {}? : {:?}",self.serial,bp_end);
            if bp_start != bp_end {
                self.bps +=1;
                log::debug!("Increasing bp count for device {} to {}",self.serial,self.bps);
                self.save_values();
            }
        }
        log::info!("Rebooting {} for the {}th time",self.serial, self.reboots);
        self.reboot();
        self.save_values();
    }
}
