from dataclasses import dataclass
import serial

@dataclass
class Device():
    def __init__(self, serial: str, gpioPort: int,
                  usbPort: serial.Serial):
        self.serial = serial
        self.gpioPort = gpioPort
        self.usbPort = usbPort
        return self

@dataclass
class DeviceLog():
    def __init__(self, device: Device, reboots: int,
                  bps: int, temps: int):
        self.device = device
        self.reboots = reboots
        self.bps = bps
        self.temps = temps
        return self
    