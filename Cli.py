#! /usr/bin/python3

from serial import Serial
import os
from pathlib import Path
import threading
import time
from Device import Device
from GPIOFacade import GPIOFacade
from queue import Queue

currentTime = time.gmtime()
timestamp = str(currentTime.tm_year) + "-" + str(currentTime.tm_mon) + "-" + str(currentTime.tm_mday) + "_" 
timestamp += str(currentTime.tm_hour) + "." + str(currentTime.tm_min) + "." + str(currentTime.tm_sec)
VERSION = "2.0.0"
iterationCount = -1
BP_CYCLES_PER_ITERATION = 3
TEMP_CYCLES_PER_ITERATION = 2

threadList = []
deviceQueue = Queue()
remainingGpioPins = set(GPIOFacade.getPins())

BAUD = 115200
COMMUNICATION_TIMEOUT = 300
TIMEOUT = 2
READ_BUFFER_SIZE = 4096
ENCODING = "utf-8"

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

def checkSerial(shell:str,queue:Queue) -> None:
    print("Testing " + shell + ". Please wait up to 30 seconds...")
    usbTTY = Serial(shell,baudrate=BAUD,timeout=COMMUNICATION_TIMEOUT)
    usbTTY.write(b'\n')
    response = usbTTY.read(READ_BUFFER_SIZE).decode(ENCODING)
    if len(response) > 0:
        queue.put(Device(usbTTY))


if __name__ == "__main__":
    print("Seymour Life V." + VERSION)

    for shellFile in Path("/dev").glob("ttyUSB*"):
        shell = os.path.join("/dev",shellFile)
        thread = threading.Thread(target=checkSerial, args=(shell, deviceQueue))
        threadList.append(thread)

    for thread in threadList: thread.start()
    for thread in threadList: thread.join()

    print(list(deviceQueue.queue))
    for device in deviceQueue.queue:
        device.darkenScreen()

    for device in deviceQueue.queue:
        device.brightenScreen()
        device.setSerial(input("Enter the serial of the device with the bright screen: "))
        device.darkenScreen()
        for pin in remainingGpioPins:
            GPIOFacade.relayHigh(pin)
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

    for device in deviceQueue.queue:
        threading.Thread(target=singleDeviceIterations, args=(device, iterationCount)).start()
