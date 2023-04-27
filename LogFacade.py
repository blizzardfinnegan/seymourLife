import logging
import datetime
now = datetime.datetime.now()
startTime = now.strftime('%Y-%m-%d_%H.%M')
formatter = logging.Formatter('%(asctime)s %(levelname)s %(message)s')

masterHandler = logging.FileHandler("logs/" + startTime + ".log")
masterHandler.setFormatter(formatter)

masterLogger = logging.getLogger("Main")
masterLogger.setLevel(logging.DEBUG)
masterLogger.addHandler(masterHandler)

class Logs: 
    def __init__(self, usbTTY: str):
        serialPort = usbTTY.split('/')[2]
        self.conciseHandler = logging.FileHandler("output/" + startTime + "-" + serialPort + ".errors")
        self.conciseHandler.setFormatter(formatter)
        self.conciseLogger = logging.getLogger("Concise")
        self.conciseLogger.setLevel(logging.WARNING)

    def debug(message) -> None:
        masterLogger.debug(message)
        masterHandler.flush()
    
    def info(message) -> None:
        masterLogger.info(message)
        masterHandler.flush()
    
    def warning(self, message) -> None:
        masterLogger.warning(message)
        masterHandler.flush()
        self.conciseLogger.warning(message)
        self.conciseHandler.flush()
    
    def error(self, message) -> None:
        masterLogger.error(message)
        masterHandler.flush()
        self.conciseLogger.error(message)
        self.conciseHandler.flush()
    
    def critical(self, message) -> None:
        masterLogger.critical(message)
        masterHandler.flush()
        self.conciseLogger.critical(message)
        self.conciseHandler.flush()
