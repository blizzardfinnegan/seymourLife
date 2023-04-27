#! /usr/bin/python3

import queue
from serial import Serial
import os
from pathlib import Path
import threading
import calendar
import glob
import time
from Device import Device
import LogFacade
from GPIOFacade import GPIOFacade

currentTime = time.gmtime()
timestamp = str(currentTime.tm_year) + "-" + str(currentTime.tm_mon) + "-" + str(currentTime.tm_mday) + "_" 
timestamp += str(currentTime.tm_hour) + "." + str(currentTime.tm_min) + "." + str(currentTime.tm_sec)
VERSION = "2.0.0"
iterationCount = -1
BP_CYCLES_PER_ITERATION = 3
TEMP_CYCLES_PER_ITERATION = 2

deviceList = []
remainingGpioPins = set(GPIOFacade.getPins())

def singleDeviceIterations(device: Device, iterationCount: int) -> None:
    for i in range(iterationCount):
        for j in range(BP_CYCLES_PER_ITERATION):
            device.startBP()
            while not (device.isBPRunning()): {}
        for j in range(TEMP_CYCLES_PER_ITERATION):
            device.startTemp()
            while not (device.isTempRunning()): {}
            device.stopTemp()
        device.reboot()
        while not (device.isRebooted()): {}

if __name__ == "__main__":
    for shellFile in Path("/dev").glob("ttyUSB*"):
        shell = os.path.join("/dev",shellFile)
        print(shellFile)
        device = Device(shell)
        deviceList.append(device)

    print(deviceList)
    for device in deviceList:
        device.darkenScreen()

    for device in deviceList:
        device.brightenScreen()
        device.setSerial(input("Enter the serial of the device with the bright screen: "))
        device.darkenScreen()
        for pin in remainingGpioPins:
            GPIOFacade.pinHigh(pin)
            time.sleep(5)
            if(device.isTempRunning()):
                device.setGPIO(pin)
        remainingGpioPins.discard(pin)

    while iterationCount < 1:
        userInput = input("Enter the number of iterations to complete: ")
        try:
            iterationCount = int(userInput)
        except:
            print("Invalid input! Please try again!")

    for device in deviceList:
        threading.Thread(target=singleDeviceIterations, args=(device, iterationCount)).start()
