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
                section['Reboots'] = deviceLog.reboots if deviceLog.reboots > section['Reboots'] else section['Reboots']
                section['Successful NIBP tests count'] = deviceLog.bps if deviceLog.bps > section['Successful NIBP tests count'] else section['Successful NIBP tests count']
                section['Successful temp test count'] = deviceLog.temps if deviceLog.temps > section['Successful temp test count'] else section['Successful temp test count']
                with open(deviceLog.device.serial + ".txt", "w") as outFile:
                    config.write(outFile)
