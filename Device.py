from enum import Enum, IntEnum, auto
from serial import Serial
import GPIOFacade
from LogFacade import Logs
import threading
import os.path

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
    LOGIN = b"root\npython3 -m debugmenu; shutdown -r now\n"
    NEWLINE = b"\n"

class SerialResponses(Enum):
    PASSWORD_PROMPT = "Password:"
    SHELL_PROMPT = "root@"
    BP_ON = "MANUAL_BP"
    BP_OFF = "IDLE"
    TEMP_FAILED = "Temp: 0"
    TEMP_SUCCESS = "Temp:"
    LOGIN_PROMPT = "login:"
    DEBUG_MENU_PROMPT = ">"
    DEBUG_CRASH = "Traceback (most recent call last)"
    DEBUG_CRASH_ERROR = "Error number -3"
    OTHER = "???"
    DECODE_ERROR = "Decode Error!"


class Device():
    def _updateOutput(self, reboots: int=0, bps:int = 0, temps: int=0) -> None:
        print("Writing output file")
        with self.lock as lock, open(self.outputFile,'w') as file:
            file.write("Reboots:" + str(reboots) + "\n")
            file.write("Successful BP cycles:" + str(bps) + "\n")
            file.write("Successful temp cycles:" + str(temps) + "\n")

    def __init__(self, usbPort: Serial, serial: str="uninitialised", gpioPin: int=-1):
        self.serial = serial
        self.outputFile = "output/" + self.serial + ".txt"
        self.gpioPin = gpioPin
        self.tty = usbPort
        print("Serial opened")
        self.state = State.LOGIN_PROMPT
        #self.logWriter = Logs(usbPort.name)
        self.reboots = 0
        self.bps = 0
        self.temps = 0
        self.lock = threading.Lock()
        if os.path.isfile(self.outputFile):
            #self.logWriter.info("Pre-existing output file! Loading, picking up where we left off.")
            with self.lock as lock, open(self.outputFile,'r') as file:
                reboots = int(file.readline().split(":")[1].strip())
                self.reboots = reboots if reboots > self.reboots else self.reboots
                bps = int(file.readline().split(":")[1].strip())
                self.bps = bps if bps > self.bps else self.bps
                temps = int(file.readline().split(":")[1].strip())
                self.temps = temps if temps > self.temps else self.temps
        else: self._updateOutput()

    def _writeToSeymour(self, command: RemoteCommand) -> None:
        print("Writing [" + str(command.value) + "] to" + self.serial)
        #self.logWriter.debug("Writing " + str(command.value) + " to device " + self.serial)
        self.tty.write(command.value)

    def _readFromSeymour(self) -> SerialResponses:
        try:
            out = self.tty.read(READ_BUFFER_SIZE).decode(ENCODING)
            #self.logWriter.info("Read [" + out + "] from device " + self.serial)
            for responseVal in SerialResponses:
                if responseVal.value in out:
                    return responseVal
            return SerialResponses.OTHER
        except UnicodeDecodeError as e:
            #self.logWriter.warning("Error with device " + self.serial + "! Failed decode of serial message.")
            return SerialResponses.DECODE_ERROR

    def _goToLoginPrompt(self) -> None:
        if not (self.state == State.LOGIN_PROMPT):
            #self.logWriter.debug("Sending device " + self.serial + " to login prompt...")
            self._writeToSeymour(RemoteCommand.QUIT)
            self.state = State.LOGIN_PROMPT

    def _goToBrightnessMenu(self) -> None:
        #self.logWriter.debug("Sending device " + self.serial + " to brightness menu...")
        while not (self.state == State.BRIGHTNESS_MENU):
            if self.state == State.BRIGHTNESS_MENU:
                #self.logWriter.info("Device " + self.serial + "already at brightness menu!")
                return
            elif self.state == State.LOGIN_PROMPT:
                self._writeToSeymour(RemoteCommand.LOGIN)
                self.state = State.DEBUG_MENU

            elif self.state == State.DEBUG_MENU:
                self._writeToSeymour(RemoteCommand.LIFECYCLE_MENU)
                self.state = State.LIFECYCLE_MENU

            elif self.state == State.LIFECYCLE_MENU:
                self._writeToSeymour(RemoteCommand.BRIGHTNESS_MENU)
                self.state = State.BRIGHTNESS_MENU

    def _goToLifecycleMenu(self) -> None:
        #self.logWriter.debug("Sending device " + self.serial + " to lifecycle menu...")
        while not (self.state == State.LIFECYCLE_MENU):
            if self.state == State.LIFECYCLE_MENU:
                #self.logWriter.info("Device " + self.serial + "already at lifecycle menu!")
                return
            
            elif self.state == State.LOGIN_PROMPT:
                self._writeToSeymour(RemoteCommand.LOGIN)
                self.state = State.DEBUG_MENU

            elif self.state == State.DEBUG_MENU:
                self._writeToSeymour(RemoteCommand.LIFECYCLE_MENU)
                self.state = State.LIFECYCLE_MENU
                return

            elif self.state == State.BRIGHTNESS_MENU:
                self._writeToSeymour(RemoteCommand.UP_MENU_LEVEL)
                self.state = State.LIFECYCLE_MENU
                return
        return
            
    def _goToDebugMenu(self) -> None:
        #self.logWriter.debug("Sending device " + self.serial + " to debug menu...")
        while not (self.state == State.DEBUG_MENU):
            if self.state == State.DEBUG_MENU:
                #self.logWriter.info("Device " + self.serial + "already at debug menu!")
                return

            elif self.state == State.LIFECYCLE_MENU:
                self._writeToSeymour(RemoteCommand.UP_MENU_LEVEL)
                self.state = State.DEBUG_MENU
                return

            elif self.state == State.LOGIN_PROMPT:
                self._writeToSeymour(RemoteCommand.LOGIN)
                self.state = State.DEBUG_MENU
                return

            elif self.state == State.BRIGHTNESS_MENU:
                self._writeToSeymour(RemoteCommand.UP_MENU_LEVEL)
                self.state = State.LIFECYCLE_MENU
        return

    def setSerial(self,serial: str) -> None:
        #self.logWriter.debug("USB Port " + self.tty.port + " connected to serial " + serial)
        #self.logWriter = Logs(self.serial)
        self._updateOutput(self.reboots,self.bps,self.temps)
        self.serial = serial

    def setGPIO(self,gpioPin: int) -> None:
        #if not (self.serial == "uninitialised"):
            #self.logWriter.debug(self.serial + " connected to GPIO pin " + gpioPin)
        #else:
            #self.logWriter.debug("USB Port " + self.tty.port + " connected to GPIO pin " + gpioPin)
        self.gpioPin = gpioPin

    def startTemp(self) -> None:
        #self.logWriter.info(self.serial + " starting temp...")
        GPIOFacade.relayHigh(self.gpioPin)

    def stopTemp(self) -> None:
        #self.logWriter.info(self.serial + " stopping temp...")
        GPIOFacade.relayLow(self.gpioPin)

    def startBP(self) -> None:
        #self.logWriter.info(self.serial + " starting BP cycle...")
        self._goToLifecycleMenu()
        self._writeToSeymour(RemoteCommand.START_BP)

    def darkenScreen(self) -> None:
        #self.logWriter.info(self.serial + " dimming screen...")
        self._goToBrightnessMenu()
        self._writeToSeymour(RemoteCommand.BRIGHTNESS_LOW)
        
    def brightenScreen(self) -> None:
        #self.logWriter.info(self.serial + " brightening screen...")
        self._goToBrightnessMenu()
        self._writeToSeymour(RemoteCommand.BRIGHTNESS_HIGH)

    def isBPRunning(self) -> bool:
        self._goToLifecycleMenu()
        self._writeToSeymour(RemoteCommand.CHECK_BP_STATE)
        #self.logWriter.info("Checking " + self.serial + " BP state:")
        while True:
            readValue = self._readFromSeymour()
            if ( readValue == SerialResponses.DEBUG_CRASH or 
                 readValue == SerialResponses.DEBUG_CRASH_ERROR or
                 readValue == SerialResponses.DECODE_ERROR):
                return self.isBPRunning()
            elif readValue == SerialResponses.OTHER:
                continue
            elif readValue == SerialResponses.BP_OFF:
                #self.logWriter.info(self.serial + ": BP off")
                return False
            elif readValue == SerialResponses.BP_ON:
                #self.logWriter.info(self.serial + ": BP on")
                self.bps += 1
                self._updateOutput(self.reboots,self.bps,self.temps)
                return True
            else:
                #if #self.logWriter is not None:
                    #self.logWriter.error(self.serial + " returned unexpected bp output! Now in unknown state!")
                #else:
                    #self.logWriter.error(self.serial + " returned unexpected bp output! Now in unknown state!")
                return False

    def isTempRunning(self) -> bool:
        self._goToLifecycleMenu()
        self._writeToSeymour(RemoteCommand.READ_TEMP)
        #self.logWriter.info("Checking " + self.serial + " BP state:")
        while True:
            readValue = self._readFromSeymour()
            if ( readValue == SerialResponses.DEBUG_CRASH or 
                 readValue == SerialResponses.DEBUG_CRASH_ERROR or
                 readValue == SerialResponses.DECODE_ERROR):
                    return self.isBPRunning()
            elif readValue == SerialResponses.OTHER:
                continue
            elif readValue == SerialResponses.TEMP_FAILED:
                return False
            elif readValue == SerialResponses.TEMP_SUCCESS:
                self.temps += 1
                self._updateOutput(self.reboots,self.bps,self.temps)
                return True
            else: 
                #if #self.logWriter is not None:
                    #self.logWriter.error(self.serial + " returned unexpected temp output! Now in unknown state!")
                #else:
                    #self.logWriter.error(self.serial + " returned unexpected temp output! Now in unknown state!")
                return False

    def reboot(self) -> None:
        #self.logWriter.info("Rebooting " + self.serial)
        self._goToLoginPrompt()
        self.reboots += 1
        self._updateOutput(self.reboots,self.bps,self.temps)
        self.state = State.LOGIN_PROMPT

    def isRebooted(self) -> bool:
        #self.logWriter.info("Checking " + self.serial + " reboot state.")
        if self.state == State.LOGIN_PROMPT:
            #self.logWriter.info(self.serial + ": rebooted successfully, awaiting login")
            return True
        else:
            #if #self.logWriter is not None:
                #self.logWriter.error(self.serial + " returned unexpected reboot output! Now in unknown state!")
            #else:
            self._goToLoginPrompt()
            self.reboots += 1
            self._updateOutput(self.reboots,self.bps,self.temps)
            return True
                #self.logWriter.error(self.serial + " returned unexpected reboot output! Now in unknown state!")
