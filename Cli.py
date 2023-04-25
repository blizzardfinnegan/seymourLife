#! /usr/bin/python3
'''
'''

import queue
import serial
import threading
import calendar
import glob
import time
import Device
import DeviceLog
import LogFacade
import GPIOFacade
import OutputFacade

currentTime = time.gmtime()
timestamp = currentTime.tm_year + "-" + currentTime.tm_mon + "-" + currentTime.tm_mday + "_" 
timestamp += currentTime.tm_hour + "." + currentTime.tm_min + "." + currentTime.tm_sec
VERSION = "2.0.0"
iterationCount = -1
BP_CYCLES_PER_ITERATION = 3
TEMP_CYCLES_PER_ITERATION = 2

gpio = GPIOFacade()
deviceList = list()
remainingGpioPins = set(gpio.getPins())
deviceMap = map()

def singleDeviceIterations(device: Device, log: DeviceLog, writer: OutputFacade, iterationCount: int) -> None:
    for i in range(iterationCount):
        for j in range(BP_CYCLES_PER_ITERATION):
            device.startBP()
            while not (device.isBPRunning()): {}
            log.successfulBP()
            writer.writeLog(log)
        for j in range(TEMP_CYCLES_PER_ITERATION):
            device.startTemp()
            while not (device.isTempRunning()): {}
            device.stopTemp()
            log.successfulTemp()
            writer.writeLog(log)
        device.reboot()
        while not (device.isRebooted()): {}
        log.successfulReboot()
        writer.writeLog(log)

if __name__ == "__main__":
    #Open Logs
    for shell in serial.tools.list_ports.comports(True):
        boolean = True
        device = Device(shell.device)
        deviceList.append(device)

    for device in deviceList:
        device.darkenScreen()
        log = DeviceLog()
        writer = OutputFacade()
        deviceMap.update({device: (log,writer)})

    for device in deviceList:
        device.brightenScreen()
        device.setSerial(input("Enter the serial of the device with the bright screen: "))
        device.darkenScreen()
        for pin in remainingGpioPins:
            gpio.pinHigh(pin)
            sleep(5)
            if(device.isTempRunning()):
                #Log pin number and the fact that its tied to the previously-set serial number
                device.setGPIO(pin)
        remainingGpioPins.discard(pin)

    #set iteration count
    for device in deviceList:
        boolean = True
        #set up threads