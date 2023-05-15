use std::{fs::{self, File}, path::Path, io::Write, thread, time::Duration};
use crate::tty::{TTY, Response,Command};
use rppal::gpio::{Gpio,OutputPin};

const BOOT_TIME:Duration = Duration::new(60, 0);
const BP_RUN:Duration = Duration::new(75, 0);
const REBOOTS_SECTION: &str = "Reboots: ";
const BP_SECTION: &str = "Successful BP tests: ";
const TEMP_SECTION: &str = "Successful temp tests: ";
const OUTPUT_FOLDER: &str = "output/";
const UNINITIALISED_SERIAL: &str = "uninitialised";
#[derive(PartialEq,Debug)]
pub enum State{
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
    bps: u64
}

impl Device{
    fn load_values(&mut self) -> &mut Self {
        if ! Path::new(&OUTPUT_FOLDER).is_dir(){
            _ = fs::create_dir(&OUTPUT_FOLDER);
        };
        log::debug!("{:?}",&self.serial);
        let output_path = OUTPUT_FOLDER.to_owned() + &self.serial + ".txt";
        if ! Path::new(&output_path).exists(){
            log::debug!("Creating file {}",output_path);
            self.output_file = Some(fs::File::create(&output_path).unwrap());
            self.save_values();
        }
        else {
            let file_contents = std::fs::read_to_string(output_path).unwrap();
            let file_lines = file_contents.split("\n");
            log::trace!("{:?}",file_contents);
            for line in file_lines {
                if line.len() > 0{
                    log::trace!("{:?}",line);
                    let section_and_data:Vec<&str> = line.split(": ").collect();
                    let section = section_and_data[0];
                    let value:u64 = section_and_data[1].parse().unwrap();
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
                };
            };
        };
        return self;
    }
    pub fn new(mut usb_port:TTY,response:Option<Response>) -> Self{
        let initial_state:State;
        match response{
            Some(response_value)=> {
                match response_value{
                    Response::PasswordPrompt=>{
                        usb_port.write_to_device(Command::Newline);
                        _ = usb_port.read_from_device(None);
                        initial_state = State::LoginPrompt;
                    },
                    Response::Other | Response::Empty | Response::ShellPrompt 
                        | Response::LoginPrompt | Response::Rebooting => 
                            initial_state = State::LoginPrompt,
                    Response::BPOn | Response::BPOff | Response::TempFailed 
                        | Response::TempSuccess =>
                            initial_state = State::LifecycleMenu,
                    Response::DebugMenuReady | Response::DebugMenuWithContinuedMessage=>
                            initial_state = State::DebugMenu,
                }
            },
            None => initial_state = State::LoginPrompt
        };
        let mut output = Self{
            usb_tty: usb_port,
            gpio: Gpio::new().unwrap(),
            address: None,
            pin: None,
            output_file: None,
            serial: UNINITIALISED_SERIAL.to_string(),
            current_state: initial_state,
            reboots: 0,
            temps: 0,
            bps: 0
        };
        output.load_values();
        return output;
    }

    fn go_to_login_prompt(&mut self) -> &mut Self{
        while !(self.current_state == State::LoginPrompt){
            match self.current_state {
                State::LoginPrompt => return self,
                State::DebugMenu | State::LifecycleMenu | State::BrightnessMenu => {
                    self.usb_tty.write_to_device(Command::Quit);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::LoginPrompt;
                    self.reboots+=1;
                    return self;
                },
            };
        };
        return self;
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
                    _ = self.usb_tty.read_from_device(None);
                    self.usb_tty.write_to_device(Command::DebugMenu);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::DebugMenu;
                },
            };
        };
        return self;
    }
    #[allow(dead_code)]
    fn go_to_debug_menu(&mut self) -> &mut Self{
        while !(self.current_state == State::DebugMenu){
            match self.current_state {
                State::DebugMenu => return self,
                State::BrightnessMenu => {
                    self.usb_tty.write_to_device(Command::UpMenuLevel);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::LifecycleMenu;
                },
                State::LifecycleMenu =>{
                    self.usb_tty.write_to_device(Command::UpMenuLevel);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::BrightnessMenu;
                },
                State::LoginPrompt => {
                    self.usb_tty.write_to_device(Command::Login);
                    _ = self.usb_tty.read_from_device(None);
                    self.usb_tty.write_to_device(Command::DebugMenu);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::DebugMenu;
                    return self;
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
                    _ = self.usb_tty.read_from_device(None);
                    self.usb_tty.write_to_device(Command::DebugMenu);
                    _ = self.usb_tty.read_from_device(None);
                    self.current_state = State::DebugMenu;
                },
            };
        };
        return self;
    }
    fn save_values(&mut self) -> &mut Self{
        let output_path = OUTPUT_FOLDER.to_owned() + &self.serial + ".txt";
        self.output_file = Some(std::fs::OpenOptions::new().write(true).truncate(true).open(output_path).unwrap());
        log::trace!("{:?}",self.output_file);
        if let Some(ref mut file_name) = self.output_file{
            log::debug!("Writing to file!");
            let mut output_data = REBOOTS_SECTION.to_string();
            output_data.push_str(&self.reboots.to_string());
            output_data.push_str("\n");
            output_data.push_str(BP_SECTION);
            output_data.push_str(&self.bps.to_string());
            output_data.push_str("\n");
            output_data.push_str(TEMP_SECTION);
            output_data.push_str(&self.temps.to_string());
            output_data.push_str("\n");
            file_name.write_all(output_data.as_bytes()).expect("Unable to write to bufwriter");
        }
        else {
            log::warn!("Cannot write to log!");
        }
        return self;
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
    pub fn set_pin_address(&mut self, address:u8) -> &mut Self{
        self.address = Some(address);
        self.pin = Some(self.gpio.get(self.address.unwrap()).unwrap().into_output());
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
    pub fn start_bp(&mut self) -> &mut Self {
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
    pub fn is_temp_running(&mut self) -> bool {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(Command::ReadTemp);
        loop {
            match self.usb_tty.read_from_device(None){
                Response::TempSuccess => return true,
                Response::TempFailed => return false,
                _ => {},
            }
        }
    }
    pub fn is_bp_running(&mut self) -> bool {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(Command::CheckBPState);
        loop { 
            match self.usb_tty.read_from_device(None){
                Response::BPOn => return true,
                Response::BPOff => return false,
                Response::DebugMenuWithContinuedMessage =>{},
                _ => return false,
            }
        }
    }
    pub fn reboot(&mut self) -> () {
        self.go_to_login_prompt();
        self.current_state = State::LoginPrompt;
    }
    pub fn is_rebooted(&mut self) -> bool {
        if self.current_state == State::LoginPrompt{
            return true;
        }
        else{
            self.go_to_login_prompt();
            self.reboots +=1;
            self.save_values();
            return true;
        }
    }
    pub fn test_cycle(&mut self, bp_cycles: Option<u64>, temp_cycles: Option<u64>) -> () {
        let local_bp_cycles: u64 = bp_cycles.unwrap_or(3);
        let local_temp_cycles: u64 = temp_cycles.unwrap_or(2);
        self.go_to_login_prompt();
        thread::sleep(BOOT_TIME);
        self.go_to_lifecycle_menu();
        //Re-open serial connection?
        _ = self.usb_tty.read_from_device(Some("["));
        for bp_count in 1..=local_bp_cycles{
            log::info!("Running bp {} on device {} ...",self.bps,self.serial);
            self.start_bp();
            let bp_start = self.is_bp_running();
            log::trace!("{:?}",bp_start);
            thread::sleep(BP_RUN);
            let bp_end = self.is_bp_running();
            log::trace!("{:?}",bp_end);
            if bp_start != bp_end {
                self.bps +=1;
                log::debug!("Increasing bp count to {}",self.bps);
                self.save_values();
            }
        }
        for temp_count in 1..=local_temp_cycles{
            log::info!("Running temp {} on device {} ...",self.temps,self.serial);
            let temp_start = self.start_temp().is_temp_running();
            let temp_end = self.stop_temp().is_temp_running();
            if temp_start != temp_end {
                self.temps +=1;
                log::debug!("Increasing temp count to {}",self.temps);
                self.save_values();
            }
        }
        log::info!("Rebooting {} for the {}th time",self.serial, self.reboots);
        self.reboot();
        self.reboots += 1;
        self.save_values();
    }
}
