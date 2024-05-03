import machine
import frame_types

from frame_types import Result_Ok
from frame_types import Result_Err
from frame_types import FrameError_NotEnoughBytes
from frame_types import FRAME_HEADER_SIZE
from frame_types import CHECKSUM_SIZE
from logging import *

RX_PIN = machine.Pin(16)
TX_PIN = machine.Pin(17)
UART2  = machine.UART(2)

UART2.init(115200, bits=8, parity=None, stop=1, tx=TX_PIN, rx=RX_PIN, timeout=10)

def read_frame_blocking(port: machine.UART):
    head = None
    
    # Read the header
    while head == None:
        head = port.read(FRAME_HEADER_SIZE)
    
    # we parse the header to see how many bytes we need to read
    (result, header) = frame_types.parse_frame_header(head)
    # log(LOGGING_LEVEL_DEBUG, f"header = {result, header}")
    if result == Result_Err:
        return Result_Err, header
    
    (_, length, _, _) = header
    
    body = None
    while body == None:
        body = port.read(length+CHECKSUM_SIZE)
        
    
    # log(LOGGING_LEVEL_DEBUG, f"Rx = {head+body}")
    
    return frame_types.parse_frame(head+body)
    
        
        
 
while True:
    result = read_frame_blocking(UART2)

    if result[0] == Result_Err:
        log(LOGGING_LEVEL_ERROR, f"Parsing error ={result}")
        exit()

    _, frame, consumed, _ = result

    tx = frame.as_bytes()
    
    log(LOGGING_LEVEL_DEBUG, f"TX = {tx}")

    UART2.write(tx)
 
 
