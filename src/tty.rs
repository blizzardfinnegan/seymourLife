use std::{collections::HashMap, io::{BufReader, BufRead, Write}, time::Duration};
use once_cell::sync::Lazy;
use serialport::{SerialPortInfo,SerialPort};
use derivative::Derivative;

pub const BAUD_RATE:u32 = 115200;

pub const AVAILABLE_TTYS: Lazy<Vec<SerialPortInfo>> = Lazy::new(||serialport::available_ports().unwrap());

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
}

#[derive(Clone,Eq,Derivative,Debug)]
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
    (Command::Login,"root\n"),
    (Command::RedrawMenu,"?"),
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
    ("login:",Response::LoginPrompt),
]));

pub struct TTY{
    tty: Box<dyn SerialPort>
}
impl std::fmt::Debug for TTY{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        f.debug_struct("TTY")
        .field("Serial port name",&self.tty.name().unwrap_or("Unknown".to_string()))
        .finish()
    }
}

impl TTY{
    pub fn new(serial_location:String) -> Self{
        if !AVAILABLE_TTYS.iter().any(|tty_info| tty_info.port_name == serial_location ) {
            panic!("Invalid TTY init string!");
        }
        else {
            return TTY { 
                tty: serialport::new(serial_location,BAUD_RATE).timeout(Duration::new(3, 0)).open().unwrap()
            };
        }
    }

    pub fn write_to_device(&mut self,command:Command) -> bool {
        println!("writing {:?} to tty {}...", command, self.tty.name().unwrap_or("unknown".to_string()));
        let output = self.tty.write_all(COMMAND_MAP.get(&command).unwrap().as_bytes()).is_ok();
        _ = self.tty.flush();
        return output;
    }

    pub fn read_from_device(&mut self,break_char:Option<&str>) -> Response {
        let mut internal_break_char = break_char.unwrap_or(">").as_bytes();
        if internal_break_char.len() == 0{
            internal_break_char = ">".as_bytes();
        }
        let mut reader = BufReader::new(&mut self.tty);
        let mut read_buffer: Vec<u8> = Vec::new();
        let read_result = reader.read_until(internal_break_char[0], &mut read_buffer).unwrap_or(0);
        println!("Successfully read {:?} from tty {}",String::from_utf8_lossy(read_buffer.as_slice()),self.tty.name().unwrap_or("unknown".to_string()));
        if read_result > 0 {
            let read_string:String = String::from_utf8_lossy(read_buffer.as_slice()).to_string();
            for possible_response in RESPONSE_MAP.keys(){
                if read_string.contains(possible_response){
                   println!("{:?} matches pattern {:?}",read_string,RESPONSE_MAP.get(*possible_response).unwrap());
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
