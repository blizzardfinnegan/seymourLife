use rppal::gpio::OutputPin;
use std::{fs::{self, File}, path::Path, io::{BufReader, BufRead}};
use crate::{tty, gpio_facade::gpio_facade};

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
    pin: Option<OutputPin>,
    serial: String,
    current_state: State,
    reboots: u64,
    temps: u64,
    bps: u64
}

impl Device{
    fn load_values(&mut self) -> () {
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

    fn go_to_login_prompt(&mut self) -> (){
        while !(self.current_state == State::LoginPrompt){
            match self.current_state {
                State::LoginPrompt => return,
                State::DebugMenu | State::LifecycleMenu | State::BrightnessMenu => {
                    self.usb_tty.write_to_device(tty::Command::Quit);
                    self.current_state = State::LoginPrompt;
                    self.reboots+=1;
                    return;
                },
            };
        };
    }

    fn go_to_brightness_menu(&mut self) -> (){
        while !(self.current_state == State::BrightnessMenu){
            match self.current_state {
                State::BrightnessMenu => return,
                State::DebugMenu => {
                    self.usb_tty.write_to_device(tty::Command::LifecycleMenu);
                    self.current_state = State::LifecycleMenu;
                    return;
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
    }
    fn go_to_debug_menu(&mut self) -> (){
        while !(self.current_state == State::DebugMenu){
            match self.current_state {
                State::DebugMenu => return,
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
                    return;
                },
            };
        };
    }
    fn go_to_lifecycle_menu(&mut self) -> (){
        while !(self.current_state == State::LifecycleMenu){
            match self.current_state {
                State::LifecycleMenu => return,
                State::DebugMenu => {
                    self.usb_tty.write_to_device(tty::Command::LifecycleMenu);
                    self.current_state = State::LifecycleMenu;
                    return;
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
    }
    pub fn set_serial(&mut self, serial:&str) -> (){}
    pub fn set_gpio(&mut self, gpio_pin: OutputPin) -> (){}
    pub fn start_temp(&mut self) -> () {}
    pub fn stop_temp(&mut self) -> () {}
    pub fn start_bp(&mut self) -> () {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(tty::Command::StartBP);
        _ = self.usb_tty.read_from_device(None);
    }
    pub fn darken_screen(&mut self) -> () {
        self.go_to_brightness_menu();
        self.usb_tty.write_to_device(tty::Command::BrightnessLow);
        _ = self.usb_tty.read_from_device(None);
        self.usb_tty.write_to_device(tty::Command::RedrawMenu);
        _ = self.usb_tty.read_from_device(None);
    }
    pub fn brighten_screen(&mut self) -> () {
        self.go_to_brightness_menu();
        self.usb_tty.write_to_device(tty::Command::BrightnessHigh);
        _ = self.usb_tty.read_from_device(None);
        self.usb_tty.write_to_device(tty::Command::RedrawMenu);
        _ = self.usb_tty.read_from_device(None);
    }
    pub fn is_temp_running(&mut self) -> bool {
        self.go_to_lifecycle_menu();
        self.usb_tty.write_to_device(tty::Command::ReadTemp);
        match self.usb_tty.read_from_device(None){
            tty::Response::TempSuccess => return true,
            _ => return false
        }
    }
    pub fn reboot(&mut self) -> () {}
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
    pub fn test_cycle(&mut self, bp_cycles: Option<u64>, temp_cycles: Option<u64>) -> () {}
}
