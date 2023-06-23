use std::{collections::HashMap, 
          io::{BufReader, Write, Read}, 
          boxed::Box,
          time::Duration};
use once_cell::sync::Lazy;
use serialport::SerialPort;
use derivative::Derivative;

const BAUD_RATE:u32 = 115200;
const SERIAL_TIMEOUT: std::time::Duration = Duration::from_millis(500);


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
    GetSerial,
}

#[derive(Clone,Eq,Derivative,Debug)]
#[derivative(PartialEq, Hash)]
pub enum Response{
    PasswordPrompt,
    ShellPrompt,
    BPOn,
    BPOff,
    TempCount(Option<u64>),
    LoginPrompt,
    DebugMenu,
    Rebooting,
    Other,
    Empty,
    ShuttingDown,
    FailedDebugMenu,
    PreShellPrompt,
    EmptyNewline,
    DebugInit,
    Serial(Option<String>),
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
    (Command::GetSerial,"echo 'y1q' | python3 -m debugmenu\n"),
]));

const RESPONSES:[(&str,Response);13] = [
    ("Last login:",Response::PreShellPrompt),
    ("reboot: Restarting",Response::Rebooting),
    ("command not found",Response::FailedDebugMenu),
    ("login:",Response::LoginPrompt),
    ("Password:",Response::PasswordPrompt),
    ("DtCtrlCfgDeviceSerialNum",Response::Serial(None)),
    ("root@",Response::ShellPrompt),
    ("EXIT Debug menu",Response::ShuttingDown),
    ("Check NIBP In Progress: True",Response::BPOn),
    ("Check NIBP In Progress: False",Response::BPOff),
    ("SureTemp Probe Pulls:",Response::TempCount(None)),
    (">",Response::DebugMenu),
    ("Loading App-Framework",Response::DebugInit),
];

pub struct TTY{
    tty: Box<dyn SerialPort>,
    last: Command,
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
        let possible_tty = serialport::new(serial_location,BAUD_RATE).timeout(SERIAL_TIMEOUT).open();
        if let Ok(tty) = possible_tty{
            Some(TTY{tty,last:Command::Quit})
        } else{
            None
        }
    }

    pub fn write_to_device(&mut self,command:Command) -> bool {
        if command == self.last{
            log::trace!("retry send {}",self.tty.name().unwrap_or("unknown".to_string()));
        }else{
            log::debug!("writing {:?} to tty {}...", command, self.tty.name().unwrap_or("unknown".to_string()));
        };
        let output = self.tty.write_all(COMMAND_MAP.get(&command).unwrap().as_bytes()).is_ok();
        self.last = command;
        _ = self.tty.flush();
        std::thread::sleep(SERIAL_TIMEOUT);
        return output;
    }

    pub fn read_from_device(&mut self,_break_char:Option<&str>) -> Response {
        let mut reader = BufReader::new(&mut self.tty);
        let mut read_buffer: Vec<u8> = Vec::new();
        _ = reader.read_to_end(&mut read_buffer);
        if read_buffer.len() > 0 {
            let read_line:String = String::from_utf8_lossy(read_buffer.as_slice()).to_string();
            if read_line.eq("\r\n") {
                return Response::EmptyNewline;
            }
            for (string,enum_value) in RESPONSES{
                if read_line.contains(string){
                    if(enum_value == Response::BPOn) || (enum_value == Response::BPOff) {
                        //Don't log BPOn or BPOff, we're gonna see a LOT of those and we don't want
                        //to overfill the SD card
                    }
                    else{
                        log::trace!("Successful read of {:?} from tty {}, which matches pattern {:?}",read_line,self.tty.name().unwrap_or("unknown shell".to_string()),enum_value);
                    };
                    if enum_value == Response::TempCount(None){
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
                                                return Response::TempCount(None)
                                            },
                                            Ok(parsed_temp_count) => {
                                                log::trace!("parsed temp count for device {}: {}",self.tty.name().unwrap_or("unknown shell".to_string()),temp_count);
                                                return Response::TempCount(Some(parsed_temp_count))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    else if enum_value == Response::Serial(None) {
                        return Response::Serial(Some(read_line));
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
            log::trace!("Unable to determine response. Response string is: [{:?}]",read_line);
            return Response::Other;
        }
        else {
            log::trace!("Read an empty string from device {:?}. Possible read error.", self);
            return Response::Empty;
        };
    }
}
