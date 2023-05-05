use std::collections::HashMap;

use serialport::{SerialPortInfo, SerialPort};

const BAUD_RATE:u32 = 115200;
const AVAILABLE_TTYS: Vec<SerialPortInfo> = serialport::available_ports();

enum State{
    LoginPrompt,
    DebugMenu,
    LifecycleMenu,
    BrightnessMenu
}

const COMMAND_MAP:HashMap<Command, = HashMap::from([
    (Command::Quit)
]);

const RESPONSE_MAP = HashMap::from([
]);
enum Command{
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
    Newline
}

enum Response{
    PasswordPrompt,
    ShellPrompt,
    BPOn,
    BPOff,
    TempFailed,
    TempSuccess,
    LoginPrompt,
    DebugMenu,
    DecodeError,
    Other
}

pub struct TTY{
    tty: Box<dyn SerialPort>,
    state: State
}

impl TTY{
    pub fn new(serial_location:String) -> Self{
    }
}

//use std::collections::HashMap;
//
//
//static mut OPEN_TTYS: HashMap<String,SerialPort> = HashMap::new();
//
//pub fn setup() -> Vec<SerialPort>{
//    let output:Vec<SerialPort> = Vec::new();
//    for tty in device::AVAILABLE_TTYS.iter(){
//        if tty.port_type == serialport::SerialPortType::UsbPort(ANY()){
//            let tty_port:Box<dyn SerialPort> = serialport::new(tty.port_name,BAUD_RATE)
//                .open().expect("Failed to open port");
//            device::OPEN_TTYS.insert(tty.port_name,*tty_port);
//            output.push(*tty_port);
//        }
//    }
//    return output;
//}
//