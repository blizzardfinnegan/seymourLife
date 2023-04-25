from enum import Enum,IntEnum, auto
from serial import Serial
import GPIOFacade

BP_TIMER = 45
TEMP_TIMER = 5

BAUD = 115200
COMMUNICATION_TIMEOUT = 300
TIMEOUT = 2
READ_BUFFER_SIZE = 4096
ENCODING = "utf-8"

class State(IntEnum):
    LOGIN_PROMPT = auto()
    DEBUG_MENU = auto()
    LIFECYCLE_MENU = auto()
    BRIGHTNESS_MENU = auto()

class RemoteCommand(Enum):
    QUIT = b"q\n"
    START_BP = b"c"
    CHECK_BP_STATE = b"C"
    LIFECYCLE_MENU = b"L"
    BRIGHTNESS_MENU = b"B"
    BRIGHTNESS_LOW = b"1"
    BRIGHTNESS_HIGH = b"0"
    READ_TEMP = b"h"
    UP_MENU_LEVEL = b"\\"
    REDRAW_MENU = "?"
    LOGIN = b"root\npython3 -m debugmenu; shutdown -r now"
    NEWLINE = b"\n"

class SerialResponses(Enum):
    PASSWORD_PROMPT = "Password:"
    SHELL_PROMPT = "root@"
    BP_ON = "MANUAL_BP"
    BP_OFF = "IDLE"
    TEMPERATURE = "Temp:"
    LOGIN_PROMPT = "login:"
    DEBUG_MENU_PROMPT = ">"
    DEBUG_MENU_EXCEPTION = "Traceback (most recent call last)"
    DEBUG_MENU_ERROR_THREE = "Error number -3"
    OTHER = "???"


class Device():
    def __init__(self, usbPort: str, serial: str="", gpioPin: int=-1):
        self.serial = serial
        self.gpioPin = gpioPin
        self.tty = Serial(usbPort,baudrate=BAUD,timeout=COMMUNICATION_TIMEOUT)
        self.state = State.LOGIN_PROMPT
        self.gpio = GPIOFacade()
        return self

    def setSerial(self,serial: str) -> None:
        self.serial = serial

    def setGPIO(self,gpioPin: int) -> None:
        self.gpioPin = gpioPin

    def _writeToSeymour(self, command: RemoteCommand) -> None:
        self.tty.write(command.value)

    def _readFromSeymour(self) -> str:
        try:
            out = self.tty.read(READ_BUFFER_SIZE).decode(ENCODING)
            return out
        except UnicodeDecodeError as e:
            #FIXME
            return ""
            #[ write to log and error ]

    def _goToLoginPrompt(self) -> None:
        if not (self.state == State.LOGIN_PROMPT):
            self._writeToSeymour(RemoteCommand.QUIT)
            self.state = State.LOGIN_PROMPT

    def _goToBrightnessMenu(self) -> None:
        while not (self.state == State.BRIGHTNESS_MENU):
            match self.state:
                case State.BRIGHTNESS_MENU:
                    return
                case State.LOGIN_PROMPT:
                    self._writeToSeymour(RemoteCommand.LOGIN)
                    self.state = State.DEBUG_MENU

                case State.DEBUG_MENU:
                    self._writeToSeymour(RemoteCommand.LIFECYCLE_MENU)
                    self.state = State.LIFECYCLE_MENU

                case State.LIFECYCLE_MENU:
                    self._writeToSeymour(RemoteCommand.BRIGHTNESS_MENU)
                    self.state = State.BRIGHTNESS_MENU

    def _goToLifecycleMenu(self) -> None:
        while not (self.state == State.LIFECYCLE_MENU):
            match self.state:
                case State.LIFECYCLE_MENU:
                    return
                
                case State.LOGIN_PROMPT:
                    self._writeToSeymour(RemoteCommand.LOGIN)
                    self.state = State.DEBUG_MENU

                case State.DEBUG_MENU:
                    self._writeToSeymour(RemoteCommand.LIFECYCLE_MENU)
                    self.state = State.LIFECYCLE_MENU
                    return

                case State.BRIGHTNESS_MENU:
                    self._writeToSeymour(RemoteCommand.UP_MENU_LEVEL)
                    self.state = State.LIFECYCLE_MENU
                    return
        return
            
    def _goToDebugMenu(self) -> None:
        while not (self.state == State.DEBUG_MENU):
            match self.state:
                case State.DEBUG_MENU:
                    return

                case State.LIFECYCLE_MENU:
                    self._writeToSeymour(RemoteCommand.UP_MENU_LEVEL)
                    self.state = State.DEBUG_MENU
                    return

                case State.LOGIN_PROMPT:
                    self._writeToSeymour(RemoteCommand.LOGIN)
                    self.state = State.DEBUG_MENU
                    return

                case State.BRIGHTNESS_MENU:
                    self._writeToSeymour(RemoteCommand.UP_MENU_LEVEL)
                    self.state = State.LIFECYCLE_MENU
        return

    def startBP(self) -> None:
        self._goToLifecycleMenu()
        self._writeToSeymour(RemoteCommand.START_BP)

    def darkenScreen(self) -> None:
        self._goToBrightnessMenu()
        self._writeToSeymour(RemoteCommand.BRIGHTNESS_LOW)
        
    def brightenScreen(self) -> None:
        self._goToBrightnessMenu()
        self._writeToSeymour(RemoteCommand.BRIGHTNESS_HIGH)

    def startTemp(self) -> None:
        self.gpio.relayHigh(self.gpioPin)

    def stopTemp(self) -> None:
        self.gpio.relayLow(self.gpioPin)

    def isBPRunning(self) -> bool:
        self._goToLifecycleMenu()
        self._writeToSeymour(RemoteCommand.CHECK_BP_STATE)
        while True:
            readString = str(self._readFromSeymour())
            if readString == "":
                raise Exception("Invalid read!")
            elif SerialResponses.BP_ON in readString: 
                return True
            elif SerialResponses.BP_OFF in readString: 
                return False

    def isTempRunning(self) -> bool:
        self._goToLifecycleMenu()
        self._writeToSeymour(RemoteCommand.READ_TEMP)
        while True:
            readString = str(self._readFromSeymour())
            if readString == "":
                raise Exception("Invalid read!")
            elif not ("Temp" in readString):
                continue
            elif "0" in readString:
                return False
            else:
                return True


    def login(self) -> bool:
        if self.state == State.LOGIN_PROMPT:
            self._writeToSeymour(RemoteCommand.LOGIN)
            self.state = State.DEBUG_MENU
            return True
        else:
            return False

    def reboot(self) -> None:
        self._goToLoginPrompt()
        self.state = State.LOGIN_PROMPT