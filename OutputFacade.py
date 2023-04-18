import configparser
import os
import threading
import queue
import DeviceLog
import ThreadSafeCounter

class OutputFacade:

    def __init__(self, queue: queue, threadCount: int) -> None:
        self.writeQueue = queue
        self.threadsRemaining = ThreadSafeCounter()
        self.threadCount = threadCount

    
    def writeThread(self) -> None:
        while self.counter.value > 0:
            config = configparser.ConfigParser()
            deviceLog = self.writeQueue.get()
            if isinstance(deviceLog, DeviceLog):
                section = config[deviceLog.device.serial]
                section['Reboots'] = deviceLog.reboots
                section['Successful NIBP tests count'] = deviceLog.bps
                section['Successful temp test count'] = deviceLog.temps
                with open(deviceLog.device.serial + ".ini", "w") as outFile:
                    config.write(outFile)