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
        self.conciseHandler = logging.FileHandler("output/" + startTime + ".errors")
        self.conciseHandler.setFormatter(formatter)
        self.conciseLogger = logging.getLogger("Concise")
        self.conciseLogger.setLevel(logging.WARNING)

    def debug(message, *args, **kwargs) -> None:
        masterLogger.debug(message,args,kwargs)
        masterHandler.flush()
    
    def info(message, *args, **kwargs) -> None:
        masterLogger.info(message,args,kwargs)
        masterHandler.flush()
    
    def warning(self, message, *args, **kwargs) -> None:
        masterLogger.warning(message,args,kwargs)
        masterHandler.flush()
        self.conciseLogger.warning(message,args,kwargs)
        self.conciseHandler.flush()
    
    def error(self, message, *args, **kwargs) -> None:
        masterLogger.error(message,args,kwargs)
        masterHandler.flush()
        self.conciseLogger.error(message,args,kwargs)
        self.conciseHandler.flush()
    
    def critical(self, message, *args, **kwargs) -> None:
        masterLogger.critical(message,args,kwargs)
        masterHandler.flush()
        self.conciseLogger.critical(message,args,kwargs)
        self.conciseHandler.flush()
