# Imports del sistema
import machine
import time
import camera
import base64
import _thread
import sys
import webserver

# Imports como namespace
import utils
import logging

# Imports al scope 
from logging import *

# Handles
EXIT_BTN  = machine.Pin(14, machine.Pin.IN, machine.Pin.PULL_UP)
BLTN_LED  = machine.Pin(33, machine.Pin.OUT)
FLSH_LED  = machine.Pin(4 , machine.Pin.OUT)
def init():
    try:
        camera.deinit()
    except:
        pass
    
    # camera.init(0, format=camera.JPEG, fb_location=camera.PSRAM)
            
    
def main():
    logging.unflushed_logs = [] # Descartamos los logs generados por el boot
    
    log(LOGGING_LEVEL_INFO, "Init...")
    init()
    
    log(LOGGING_LEVEL_INFO, "Init success")
    
    web_server = webserver.webcam()
    web_server.run(config.app_config)
    
    
if __name__ == "__main__":
    main()
