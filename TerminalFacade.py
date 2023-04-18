from enum import Enum

BP_TIMER = 45
TEMP_TIMER = 5
RESPONSE_TIMEOUT = 180
REBOOT_TIMEOUT = 300

BAUD = 115200
TIMEOUT = 2
READ_BUFFER_SIZE = 4096
ENCODING = "utf-8"
PORT_LIST = list(f"/dev/ttyUSB{i}" for i in range(10))

class DebugMenuCommands(Enum):
    QUIT = b"q\n"
    BP_MENU = b"n"
    START_BP = b"s"
    CANCEL_BP = b"c"
    CHECK_STATE = b"C"
    LIFECYCLE_MENU = b"L"
    BRIGHTNESS_MENU = b"B"
    BRIGHTNESS_LOW = b"1"
    BRIGHTNESS_HIGH = b"0"
    READ_TEMP = b"h"
    UP_MENU_LEVEL = b"\\"
    REDRAW_MENU = "?"

class LinuxCommands(Enum):
    LOGIN = b"root\n"
    LOGOUT = b"logout\n"
    REBOOT = b"shutdown -r now\n"
    NEWLINE = b"\n"
    ENTER_DEBUG_MENU = b"python3 -m debugmenu\n"
    STOP_BP_DAEMON = b"systemctl stop wa_nipd\n"
    START_BP_DAEMON = b"systemctl start wa_nipd\n"
    GPIO_HIGH = b"gpioset 2 22=1\n"

class SerialResponses(Enum):
    PASSWORD_PROMPT = "Password:"
    SHELL_PROMPT = "root@"
    BP_START = "MANUAL_BP"
    BP_FINISH = "IDLE"
    TEMPERATURE = "Temp:"
    LOGIN_PROMPT = "login:"
    DEBUG_MENU_PROMPT = ">"
    DEBUG_MENU_EXCEPTION = "Traceback (most recent call last)"
    DEBUG_MENU_ERROR_THREE = "Error number -3"

