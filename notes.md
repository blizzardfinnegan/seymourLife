Auto-serial:

`echo 'y1q' | python3 -m debugmenu` contains the serial number. Search keyword is: "DtCtrlCfgDeviceSerialNum"
split on `\n`, collect into Vec<&str>. Iterate over Vec:
    if doesn't contain colon, continue
    split_once on colon
    if first half is keyword, trim second half, remove `"` characters, save as serial
