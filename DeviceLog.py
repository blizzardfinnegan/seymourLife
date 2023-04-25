from dataclasses import dataclass
import serial

@dataclass

@dataclass
class DeviceLog():
    def __init__(self, device: Device, reboots: int,
                  bps: int, temps: int):
        self.device = device
        self.reboots = reboots
        self.bps = bps
        self.temps = temps
        return self
    