from threading import Lock

class ThreadSafeCounter():
    # constructor
    def __init__(self, initialSize: int=0):
        # initialize counter
        self._counter = initialSize
        # initialize lock
        self._lock = Lock()
 
    # increment the counter
    def increment(self):
        with self._lock:
            self._counter += 1

     # decrement the counter
    def decrement(self):
        with self._lock:
            self._counter -= 1

    # get the counter value
    def value(self):
        with self._lock:
            return self._counter