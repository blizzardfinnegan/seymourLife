Boot time isn't static.
Boot time can (in theory) be sped up by forcing a clearing of the screen of all vitals before rebooting.
Clearing the screen of all vitals is theoretically possible in the UI menu, with a Spoof Touch, but I don't know the object name of the 'clear' button.

Can check for reboot completed by reading in serial, check for `reboot: Restarting System`
once done, read for `login:`

First command sent to shell gets dropped (possibly due to timing issue?)
