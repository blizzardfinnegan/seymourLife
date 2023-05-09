use std::{collections::HashMap, io::{BufReader, BufRead}};
use once_cell::sync::Lazy;
use serialport::{SerialPortInfo, SerialPort};
use derivative::Derivative;

const BAUD_RATE:u32 = 115200;

const AVAILABLE_TTYS: Lazy<Vec<SerialPortInfo>> = Lazy::new(||serialport::available_ports().unwrap());

#[derive(Eq,Derivative)]
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
}

#[derive(Clone,Eq,Derivative)]
#[derivative(Copy,PartialEq, Hash)]
pub enum Response{
    PasswordPrompt,
    ShellPrompt,
    BPOn,
    BPOff,
    TempFailed,
    TempSuccess,
    LoginPrompt,
    DebugMenu,
    Rebooting,
    Other,
}


const COMMAND_MAP:Lazy<HashMap<Command,&str>> = Lazy::new(||HashMap::from([
    (Command::Quit, "q\n"),
    (Command::StartBP, "c"),
    (Command::CheckBPState, "C"),
    (Command::LifecycleMenu, "L"),
    (Command::BrightnessMenu, "B"),
    (Command::BrightnessHigh, "0"),
    (Command::BrightnessLow, "1"),
    (Command::ReadTemp, "h"),
    (Command::UpMenuLevel, "\\"),
    (Command::Login,"root"),
    (Command::DebugMenu,"python3 -m debugmenu; shutdown -r now\n"),
    (Command::Newline,"\n"),
]));

const RESPONSE_MAP:Lazy<HashMap<&str,Response>> = Lazy::new(||HashMap::from([
    ("Password:",Response::PasswordPrompt),
    ("root@",Response::ShellPrompt),
    ("Check NIBP In Progress: True",Response::BPOn),
    ("Check NIBP In Progress: False",Response::BPOff),
    ("Temp: 0",Response::TempFailed),
    ("Temp:",Response::TempSuccess),
    (">",Response::DebugMenu),
    ("[",Response::Rebooting),
]));

pub struct TTY{
    tty: Box<dyn SerialPort>
}

impl TTY{
    pub fn new(serial_location:String) -> Self{
        if !AVAILABLE_TTYS.iter().any(|tty_info| tty_info.port_name == serial_location ) {
            panic!("Invalid TTY init string!");
        }
        else {
            return TTY { 
                tty: serialport::new(serial_location, BAUD_RATE).open().unwrap()
            };
        }
    }

    pub fn write_to_device(&mut self,command:Command) -> bool {
        return self.tty.write(COMMAND_MAP.get(&command).unwrap().as_bytes()).unwrap() > 0;
    }

    pub fn read_from_device(&mut self,break_char:Option<&str>) -> Response {
        let internal_break_char = break_char.unwrap_or(">").as_bytes()[0];
        let mut reader = BufReader::new(&mut self.tty);
        let mut read_buffer: Vec<u8> = Vec::new();
        let read_result = reader.read_until(internal_break_char, &mut read_buffer).unwrap_or(0);
        if read_result > 0 {
            let read_string:String = String::from_utf8_lossy(read_buffer.as_slice()).to_string();
            for possible_response in RESPONSE_MAP.keys(){
                if read_string.contains(possible_response){
                    return *RESPONSE_MAP.get(*possible_response).unwrap_or(&Response::Other);
                }
            }
            return Response::Other;
        }
        else {
            return Response::Other;
        };
    }
}
