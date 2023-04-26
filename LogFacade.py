import logging
import datetime
now = datetime.datetime.now()
startTime = now.strftime('%Y-%m-%d_%H.%M')
formatter = logging.Formatter('%(asctime)s %(levelname)s %(message)s')

masterHandler = logging.fileHandler("logs/" + startTime + ".log")
masterHandler.setFormatter(formatter)

masterLogger = logging.getLogger("Main")
masterLogger.setLevel(logging.DEBUG)
masterLogger.addHandler(masterHandler)

class Logs: 
    def __init__(self, serial: str):
        self.conciseHandler = logging.fileHandler("output/" + startTime + ".errors")
        self.conciseHandler.setFormatter(formatter)
        self.conciseLogger = logging.getLogger("Concise")
        self.conciseLogger.setLevel(logging.WARNING)

    def debug(message, *args, **kwargs) -> None:
        masterLogger.debug(message,args,kwargs)
    
    def info(message, *args, **kwargs) -> None:
        masterLogger.info(message,args,kwargs)
    
    def warning(self, message, *args, **kwargs) -> None:
        masterLogger.warning(message,args,kwargs)
        self.conciseLogger.warning(message,args,kwargs)
    
    def error(self, message, *args, **kwargs) -> None:
        masterLogger.error(message,args,kwargs)
        self.conciseLogger.error(message,args,kwargs)
    
    def critical(self, message, *args, **kwargs) -> None:
        masterLogger.critical(message,args,kwargs)
        self.conciseLogger.critical(message,args,kwargs)