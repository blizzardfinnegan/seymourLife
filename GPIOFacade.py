import RPi.GPIO as GPIO

class GPIOFacade:
    # Below code will always run first.
    RELAY_PINS = [4,5,6,12,13,17,18,19,20,26]
    GPIO.setmode(GPIO.BCM)
    GPIO.setwarnings(False)
    for pin in RELAY_PINS:
        GPIO.setup(pin, GPIO.OUT)
        GPIO.output(pin, GPIO.LOW)

    def __init__(self):
        return self

    def relayHigh(pin: int) -> None:
        if pin in GPIOFacade.RELAY_PINS:
            GPIO.output(pin,GPIO.HIGH)
        else:
            raise Exception("Pin " + pin + " is invalid pin!")

    def relayLow(pin: int) -> None:
        if pin in GPIOFacade.RELAY_PINS:
            GPIO.output(pin,GPIO.LOW)
        else:
            raise Exception("Pin " + pin + " is invalid pin!")

    def close() -> None:
        for pin in GPIOFacade.RELAY_PINS: GPIO.output(pin,GPIO.LOW)

    def getPins():
        return GPIOFacade.RELAY_PINS
