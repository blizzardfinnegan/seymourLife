#! /usr/bin/python3

import queue
import serial
import threading
import calendar
import glob
import time
import Device
import LogFacade
import GPIOFacade

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
    for shell in serial.tools.list_ports.comports(True):
        boolean = True
        device = Device(shell.device)
        deviceList.append(device)

    for device in deviceList:
        device.darkenScreen()

    for device in deviceList:
        device.brightenScreen()
        device.setSerial(input("Enter the serial of the device with the bright screen: "))
        device.darkenScreen()
        for pin in remainingGpioPins:
            gpio.pinHigh(pin)
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