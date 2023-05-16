use std::{collections::HashMap, io::{BufReader, Write, Read}, time::Duration};
use once_cell::sync::Lazy;
use serialport::{SerialPortInfo,SerialPort};
use derivative::Derivative;

const BAUD_RATE:u32 = 115200;
const SERIAL_READ_TIMEOUT: std::time::Duration = Duration::from_millis(500);

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
    DebugMenuReady,
    DebugMenuWithContinuedMessage,
    Rebooting,
    Other,
    Empty,
}


const COMMAND_MAP:Lazy<HashMap<Command,&str>> = Lazy::new(||HashMap::from([
    (Command::Quit, "q\n"),
    (Command::StartBP, "N"),
    (Command::CheckBPState, "n"),
    (Command::LifecycleMenu, "L"),
    (Command::BrightnessMenu, "B"),
    (Command::BrightnessHigh, "0"),
    (Command::BrightnessLow, "1"),
    (Command::ReadTemp, "h"),
    (Command::UpMenuLevel, "\\"),
    (Command::Login,"root\n"),
    (Command::RedrawMenu,"?"),
    (Command::DebugMenu," python3 -m debugmenu; shutdown -r now\n"),
    (Command::Newline,"\n"),
]));

const RESPONSES:[(&str,Response);10] = [
    ("login:",Response::LoginPrompt),
    ("Password:",Response::PasswordPrompt),
    ("root@",Response::ShellPrompt),
    ("Check NIBP In Progress: True",Response::BPOn),
    ("Check NIBP In Progress: False",Response::BPOff),
    ("Temp: 0",Response::TempFailed),
    ("Temp:",Response::TempSuccess),
    ("> ",Response::DebugMenuWithContinuedMessage),
    (">",Response::DebugMenuReady),
    ("[",Response::Rebooting),
];

pub struct TTY{
    tty: Box<dyn SerialPort>,
    failed_read_count: u8
}
impl std::fmt::Debug for TTY{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result{
        f.debug_struct("TTY")
        .field("Serial port name",&self.tty.name().unwrap_or("Unknown".to_string()))
        .finish()
    }
}

impl TTY{
    pub fn new(serial_location:&str) -> Self{
            TTY { 
                tty: serialport::new(serial_location,BAUD_RATE).timeout(SERIAL_READ_TIMEOUT).open().expect("Unable to open serial connnection!"),
                failed_read_count: 0
            }
    }

    pub fn write_to_device(&mut self,command:Command) -> bool {
        log::debug!("writing {:?} to tty {}...", command, self.tty.name().unwrap_or("unknown".to_string()));
        let output = self.tty.write_all(COMMAND_MAP.get(&command).unwrap().as_bytes()).is_ok();
        _ = self.tty.flush();
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
                   log::debug!("Successful read of {:?} from tty {}, which matches pattern {:?}",read_line,self.tty.name().unwrap_or("unknown shell".to_string()),enum_value);
                   self.failed_read_count = 0;
                    return enum_value;
                }
            }
            return Response::Other;
        }
        else {
            log::debug!("Read an empty string. Possible read error.");
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
