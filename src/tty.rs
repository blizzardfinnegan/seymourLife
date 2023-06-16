use std::{collections::HashMap, io::{BufReader, Write, Read}, time::Duration};
use once_cell::sync::Lazy;
use serialport::SerialPort;
use derivative::Derivative;

const BAUD_RATE:u32 = 115200;
const SERIAL_READ_TIMEOUT: std::time::Duration = Duration::from_millis(500);


#[derive(Eq,Derivative,Debug)]
#[derivative(PartialEq, Hash)]
pub enum Command{
    Quit,
    StartBP,
    CheckBPState,
    LifecycleMenu,
    BrightnessMenu,
    BrightnessLow,
    BrightnessHigh,
    ReadTemp,
    UpMenuLevel,
    RedrawMenu,
    Login,
    DebugMenu,
    Newline,
    Shutdown,
}

#[derive(Clone,Eq,Derivative,Debug)]
#[derivative(Copy,PartialEq, Hash)]
pub enum Response{
    PasswordPrompt,
    ShellPrompt,
    BPOn,
    BPOff,
    TempCount(u64),
    LoginPrompt,
    DebugMenu,
    Rebooting,
    Other,
    Empty,
    ShuttingDown,
    FailedDebugMenu,
}


const COMMAND_MAP:Lazy<HashMap<Command,&str>> = Lazy::new(||HashMap::from([
    (Command::Quit, "q\n"),
    (Command::StartBP, "N"),
    (Command::CheckBPState, "n"),
    (Command::LifecycleMenu, "L"),
    (Command::BrightnessMenu, "B"),
    (Command::BrightnessHigh, "0"),
    (Command::BrightnessLow, "1"),
    (Command::ReadTemp, "H"),
    (Command::UpMenuLevel, "\\"),
    (Command::Login,"root\n"),
    (Command::RedrawMenu,"?"),
    (Command::DebugMenu," python3 -m debugmenu; shutdown -r now\n"),
    (Command::Newline,"\n"),
    (Command::Shutdown,"shutdown -r now\n"),
]));

const RESPONSES:[(&str,Response);10] = [
    ("reboot: Restarting",Response::Rebooting),
    ("command not found",Response::FailedDebugMenu),
    ("login:",Response::LoginPrompt),
    ("Password:",Response::PasswordPrompt),
    ("EXIT Debug menu",Response::ShuttingDown),
    ("root@",Response::ShellPrompt),
    ("Check NIBP In Progress: True",Response::BPOn),
    ("Check NIBP In Progress: False",Response::BPOff),
    ("SureTemp Probe Pulls:",Response::TempCount(0)),
    (">",Response::DebugMenu),
];

pub struct TTY{
    tty: Box<dyn SerialPort>,
    failed_read_count: u8
}
impl std::fmt::Debug for TTY{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        let absolute_location = self.tty.name();
        let relative_location:String;
        match absolute_location{
            Some(abs_location_string) => {
                let sectioned_abs_location = abs_location_string.rsplit_once('/');
                match sectioned_abs_location{
                    Some((_,serial_device_name)) => relative_location = serial_device_name.to_string(),
                    None => relative_location = "unknown".to_string()
                }
            },
            None => relative_location = "unknown".to_string()
        };
        f.debug_struct("TTY")
        .field("Serial port name",&relative_location)
        .finish()
    }
}

impl TTY{
    pub fn new(serial_location:&str) -> Option<Self>{
        let possible_tty = serialport::new(serial_location,BAUD_RATE).timeout(SERIAL_READ_TIMEOUT).open();
        if let Ok(tty) = possible_tty{
            Some(TTY { 
                tty,
                failed_read_count: 0
            })
        } else{
            None
        }
    }

    pub fn write_to_device(&mut self,command:Command) -> bool {
        log::trace!("writing {:?} to tty {}...", command, self.tty.name().unwrap_or("unknown".to_string()));
        let output = self.tty.write_all(COMMAND_MAP.get(&command).unwrap().as_bytes()).is_ok();
        _ = self.tty.flush();
        if command == Command::Login { std::thread::sleep(std::time::Duration::from_secs(2)); }
        std::thread::sleep(std::time::Duration::from_millis(500));
        return output;
    }

    pub fn read_from_device(&mut self,_break_char:Option<&str>) -> Response {
        let mut reader = BufReader::new(&mut self.tty);
        let mut read_buffer: Vec<u8> = Vec::new();
        _ = reader.read_to_end(&mut read_buffer);
        if read_buffer.len() > 0 {
            let read_line:String = String::from_utf8_lossy(read_buffer.as_slice()).to_string();
            for (string,enum_value) in RESPONSES{
                if read_line.contains(string){
                   log::trace!("Successful read of {:?} from tty {}, which matches pattern {:?}",read_line,self.tty.name().unwrap_or("unknown shell".to_string()),enum_value);
                   self.failed_read_count = 0;
                    if enum_value == Response::TempCount(0){
                        let mut lines = read_line.lines();
                        while let Some(single_line) = lines.next(){
                            if single_line.contains(string){
                                let trimmed_line = single_line.trim();
                                match trimmed_line.rsplit_once(' '){
                                    None =>  return enum_value,
                                    Some((_header,temp_count)) => {
                                        match temp_count.trim().parse::<u64>(){
                                            Err(_) => {
                                                log::error!("String {} from device {} unable to be parsed!",temp_count,self.tty.name().unwrap_or("unknown shell".to_string()));
                                                return Response::TempCount(0)
                                            },
                                            Ok(parsed_temp_count) => {
                                                //log::trace!("Header: {}",header);
                                                log::trace!("parsed temp count for device {}: {}",self.tty.name().unwrap_or("unknown shell".to_string()),temp_count);
                                                return Response::TempCount(parsed_temp_count)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    else if enum_value == Response::PasswordPrompt {
                        log::error!("Recieved password prompt on device {}! Something fell apart here. Check preceeding log lines.",self.tty.name().unwrap_or("unknown shell".to_string()));
                        self.write_to_device(Command::Newline);
                        _ = self.read_from_device(None);
                    }
                    else{
                        return enum_value;
                    }
                }
            }
            return Response::Other;
        }
        else {
            log::debug!("Read an empty string from device {:?}. Possible read error.", self);
            //Due to a linux kernel power-saving setting that is overly complicated to fix,
            //Serial connections will drop for a moment before re-opening, at seemingly-random
            //intervals. The below is an attempt to catch and recover from this behaviour.
            self.failed_read_count += 1;
            if self.failed_read_count >= 15{
                self.failed_read_count = 0;
                let tty_location = self.tty.name().expect("Unable to read tty name!");
                self.tty = serialport::new(tty_location,BAUD_RATE).timeout(SERIAL_READ_TIMEOUT).open().expect("Unable to open serial connection!");
                return self.read_from_device(_break_char);
            }
            return Response::Empty;
        };
    }
}
