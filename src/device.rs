use std::{fs::{self, File}, path::Path, io::{BufReader, BufRead}, thread, time::Duration};
use crate::{tty, gpio_facade::Relay};

const BOOT_TIME:Duration = Duration::new(60, 0);
const BP_START:Duration = Duration::new(60, 0);
const BP_RUN:Duration = Duration::new(60, 0);
#[derive(PartialEq)]
pub enum State{
    LoginPrompt,
    DebugMenu,
    LifecycleMenu,
    BrightnessMenu
}

pub struct Device{
    usb_tty: tty::TTY,
    output_file: Option<File>,
    pin: Option<Relay>,
    serial: String,
    current_state: State,
    reboots: u64,
    temps: u64,
    bps: u64
}

impl Device{
    fn load_values(&mut self) -> &mut Self {
        let mut output_path:String = "output/".to_string();
        if ! Path::new(&output_path).is_dir(){
            _ = fs::create_dir(&output_path);
        };
        output_path.push_str(&self.serial.to_string());
        if ! Path::new(&output_path).exists(){
            self.output_file = Some(fs::File::create(&output_path).unwrap());
        }
        else {
            if let Some(file) = &self.output_file{
                let reader = BufReader::new(file);
                for line in reader.lines(){
                    let unwrapped_line = line.unwrap().to_string();
                    let mut split_sections = unwrapped_line.split(": ");
                    let section: &str = &*split_sections.next().unwrap().to_lowercase();
                    let value = split_sections.next().unwrap().parse::<u64>().unwrap();
                    match section {
                        "Reboots: " => {
                            self.reboots = value;
                        },
                        "Successful BP cycles:" => {
                            self.bps = value;
                        },
                        "Successful temp cycles:" => {
                            self.temps = value;
                        },
                        _ => ()
                    };
                };
            }
        };
        return self;
    }
    pub fn new(usb_port:tty::TTY) -> Self{
        let mut output = Self{
            usb_tty: usb_port,
            pin: None,
            output_file: None,
            serial: "uninitialised".to_string(),
            current_state: State::LoginPrompt,
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
                    self.usb_tty.write_to_device(tty::Command::Quit);
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
                    self.usb_tty.write_to_device(tty::Command::LifecycleMenu);
                    self.current_state = State::LifecycleMenu;
                    return self;
                },
                State::LifecycleMenu =>{
                    self.usb_tty.write_to_device(tty::Command::BrightnessMenu);
                    self.current_state = State::BrightnessMenu;
                },
                State::LoginPrompt => {
                    self.usb_tty.write_to_device(tty::Command::Login);
                    self.usb_tty.write_to_device(tty::Command::DebugMenu);
                    self.current_state = State::DebugMenu;
                },
            };
        };
        return self;
    }
    fn go_to_debug_menu(&mut self) -> &mut Self{
        while !(self.current_state == State::DebugMenu){
            match self.current_state {
                State::DebugMenu => return self,
                State::BrightnessMenu => {
                    self.usb_tty.write_to_device(tty::Command::UpMenuLevel);
                    self.current_state = State::LifecycleMenu;
                },
                State::LifecycleMenu =>{
                    self.usb_tty.write_to_device(tty::Command::UpMenuLevel);
                    self.current_state = State::BrightnessMenu;
                },
                State::LoginPrompt => {
                    self.usb_tty.write_to_device(tty::Command::Login);
                    self.usb_tty.write_to_device(tty::Command::DebugMenu);
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
                    self.usb_tty.write_to_device(tty::Command::LifecycleMenu);
                    self.current_state = State::LifecycleMenu;
                    return self;
                },
                State::BrightnessMenu =>{
                    self.usb_tty.write_to_device(tty::Command::UpMenuLevel);
                    self.current_state = State::BrightnessMenu;
                },
                State::LoginPrompt => {
                    self.usb_tty.write_to_device(tty::Command::Login);
                    self.usb_tty.write_to_device(tty::Command::DebugMenu);
                    self.current_state = State::DebugMenu;
                },
            };
        };
        return self;
    }
    pub fn set_serial(&mut self, serial:&str) -> &mut Self{
        self.serial = serial.to_string();
        self.load_values();
        return self;
    }
    pub fn set_gpio(&mut self, gpio_pin: Relay) -> &mut Self{
        self.pin = Some(gpio_pin);
        return self;
    }
    pub fn start_temp(&mut self) -> &mut Self {
        if let Some(ref mut gpio_pin) = self.pin{
            gpio_pin.high();
        }
        return self;
    }
    pub fn stop_temp(&mut self) -> &mut Self {
        if let Some(ref mut gpio_pin) = self.pin{
            gpio_pin.low();
        }
        return self;
    }
    pub fn start_bp(&mut self) -> &mut Self {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(tty::Command::StartBP);
        _ = self.usb_tty.read_from_device(None);
        return self;
    }
    pub fn darken_screen(&mut self) -> &mut Self {
        self.go_to_brightness_menu();
        self.usb_tty.write_to_device(tty::Command::BrightnessLow);
        _ = self.usb_tty.read_from_device(None);
        self.usb_tty.write_to_device(tty::Command::RedrawMenu);
        _ = self.usb_tty.read_from_device(None);
        return self;
    }
    pub fn brighten_screen(&mut self) -> &mut Self {
        self.go_to_brightness_menu();
        self.usb_tty.write_to_device(tty::Command::BrightnessHigh);
        _ = self.usb_tty.read_from_device(None);
        self.usb_tty.write_to_device(tty::Command::RedrawMenu);
        _ = self.usb_tty.read_from_device(None);
        return self;
    }
    pub fn is_temp_running(&mut self) -> bool {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(tty::Command::ReadTemp);
        match self.usb_tty.read_from_device(None){
            tty::Response::TempSuccess => return true,
            _ => return false
        }
    }
    pub fn is_bp_running(&mut self) -> bool {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(tty::Command::CheckBPState);
        match self.usb_tty.read_from_device(None){
            tty::Response::BPOn => return true,
            _ => return false
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
            //Write values to file
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
            log::info!("Running bp {} on device {} ...",bp_count,self.serial);
            _ = self.start_bp().usb_tty.read_from_device(None);
            thread::sleep(BP_START);
            let _bp_start = self.is_bp_running();
            thread::sleep(BP_RUN);
            let _bp_end = self.is_bp_running();
            if _bp_start != _bp_end {
                self.bps +=1;
                //Write values to file
            }
        }
        for temp_count in 1..=local_temp_cycles{
            log::info!("Running temp {} on device {} ...",temp_count,self.serial);
            let _temp_start = self.start_temp().is_temp_running();
            let _temp_end = self.stop_temp().is_temp_running();
            if _temp_start != _temp_end {
                self.temps +=1;
                //Write values to file
            }
        }
        log::info!("Rebooting {}",self.serial);
        self.reboot();
        self.reboots += 1;
        //Write values to file
    }
}
