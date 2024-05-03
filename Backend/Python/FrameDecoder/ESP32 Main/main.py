

# Imports del sistema
import machine
import time
import math
import network

# Imports como namespace
import frame_ops
import frame_types
import procs
import sensores
import utils
import Stepper



# Imports al scope
from logging import *
from state import State
from frame_stack import FrameStack
from frame_types import Result_Err



# Constantes
V_GEAR_RATIO = 8.0 / 1.0
H_GEAR_RATIO = 8.0 / 1.0
VERTICAL_ANGLE_RANGE   = (10.0, 90.0)
HORIZONTAL_ANGLE_RANGE = (0.0 , 360.0)

# Pines de conexión
UART2_RX_PIN  = 16
UART2_TX_PIN  = 17
ACCEL_SCL_PIN = 22 # No puede cambiar sin alterar la librería
ACCEL_SDA_PIN = 21 # No puede cambiar sin alterar la librería
V_STEPPER     = [2 , 4 , 18, 19]
H_STEPPER     = [13, 12, 14, 27]


# Handles
UART2     = machine.UART(2)
ACCEL     = sensores.ACCEL(sda=ACCEL_SDA_PIN, scl=ACCEL_SCL_PIN)
V_STEPPER = Stepper.Stepper(mode="HALF_STEP", pin1=V_STEPPER[0], pin2=V_STEPPER[1], pin3=V_STEPPER[2], pin4=V_STEPPER[3], delay=5)
H_STEPPER = Stepper.Stepper(mode="HALF_STEP", pin1=H_STEPPER[0], pin2=H_STEPPER[1], pin3=H_STEPPER[2], pin4=H_STEPPER[3], delay=5)
nic       = network.WLAN(network.STA_IF)



# Información de protocolo de transmisión de tramas
frame_stack     = FrameStack()

# Callbacks
def onPositionChange(state: State, old_pitch, old_yaw):
    log(LOGGING_LEVEL_DEBUG, "onPositionChange")
    
    delta_pitch = delta_yaw = 0
    sign_pitch  = sign_yaw  = 1
    
    if state.known_position:
        from frame_types import Position
        log(LOGGING_LEVEL_INFO, "Moving with known position...")
        
        delta_pitch = state.position.pitch - old_pitch
        delta_yaw   = state.position.yaw   - old_yaw
        
        sign_pitch  = int(math.copysign(1, delta_pitch))
        sign_yaw    = int(math.copysign(1, delta_yaw))
        
        delta_pitch = Position.raw_as_deg(abs(delta_pitch))
        delta_yaw   = Position.raw_as_deg(abs(delta_yaw  ))
        
        log(LOGGING_LEVEL_VERBOSE, f"delta_pitch={delta_pitch}, delta_yaw={delta_yaw}")
    
    else:
        log(LOGGING_LEVEL_INFO, "Measuring current pitch...")
        acc, sz = 0, 10
        
        for i in range(sz):
            (acc_x, _, _, _, _, _) = ACCEL.measure()
            acc += utils.angle_from_gravity(acc_x)
            time.sleep_ms(5)
            
        avg_pitch = acc / sz
        req_pitch = state.position.pitch_as_deg()
        log(LOGGING_LEVEL_INFO, f"measured pitch = {avg_pitch}°, requeuested pitch = {req_pitch}")
        
        delta_pitch = req_pitch - avg_pitch
        state.known_position = True
        

    log(LOGGING_LEVEL_INFO, f"Moving vertically {sign_pitch * delta_pitch}°, ratio={V_GEAR_RATIO}, total={delta_pitch * V_GEAR_RATIO}°")
    V_STEPPER.angle(delta_pitch * V_GEAR_RATIO, direction=sign_pitch)
    
    log(LOGGING_LEVEL_INFO, f"Moving horizontally {sign_yaw * delta_yaw}°, ratio={H_GEAR_RATIO}, total={delta_yaw * H_GEAR_RATIO}°")
    H_STEPPER.angle(delta_yaw   * H_GEAR_RATIO, direction=sign_yaw)
    
# Parámetros de estado físico
state = State(onPositionChange)


# Funcionalidad específica a esta ESP
def advance_step(state: State, frame_stack: FrameStack, port: machine.UART):
    pitch_step, yaw_step = state.step_size.pitch_step, state.step_size.yaw_step
    pitch     , yaw      = state.position.pitch      , state.position.yaw
    
    log(LOGGING_LEVEL_VERBOSE, f"curr_pitch={pitch}, pitch_step={pitch_step}")

    new_pitch = pitch + pitch_step * state.direction.vertical
    new_pitch_deg = frame_types.Position.raw_as_deg(new_pitch)
    
    log(LOGGING_LEVEL_DEBUG, f"next_pitch = {new_pitch_deg}°")
    if VERTICAL_ANGLE_RANGE[0] - 1 <= new_pitch_deg <= VERTICAL_ANGLE_RANGE[1] + 1:
        # Esta dentro del rango vertical
        log(LOGGING_LEVEL_DEBUG, f"Moving only vertically, pitch = {frame_types.Position.raw_as_deg(pitch)}° -> {frame_types.Position.raw_as_deg(new_pitch)}")
        pitch = new_pitch
    
    else:
        # Esta fuera de rango. Invertimos la dirección vertical y avanzamos un paso horizontal
        state.direction.invert_vertical()
        
        new_yaw = yaw + yaw_step * state.direction.horizontal
        new_yaw_deg = frame_types.Position.raw_as_deg(new_yaw)
        
        log(LOGGING_LEVEL_DEBUG, f"next_yaw = {new_yaw_deg}°")
        if HORIZONTAL_ANGLE_RANGE[0] <= new_yaw_deg <= HORIZONTAL_ANGLE_RANGE[1]:
            # Está dentro del rango horizontal
            log(LOGGING_LEVEL_DEBUG, f"Moving only Horizontally, yaw = {frame_types.Position.raw_as_deg(yaw)}° -> {frame_types.Position.raw_as_deg(new_yaw)}")
            yaw += yaw_step
        
        else:
            # Ya terminamos. Regresamos a la posición original para desenrredar los cables y emitimos un EoT
            log(LOGGING_LEVEL_DEBUG, "Bob's your uncle.")
            frame_ops.tx_new_frame(frame_types.Cmd_EndOfTransmission, {}, frame_stack, port)
            state.active_connection = False
            state.direction.invert_horizontal()
            state.set_position(pitch, 0)
            state.direction.invert_horizontal()
    
    
    state.set_position(pitch, yaw)
    

def measure_rssi(state: State):

    weight = 1 / state.measurements_per_step
    
    RSSIs      = dict() # BSSID -> rssi
    records    = state.pending_records
    new_SSIDs  = state.pending_ssids
    new_BSSIDs = state.pending_bssids
    
    for _ in range(state.measurements_per_step):
        for item in nic.scan():
            ssid        = frame_types.SSID (item[0])
            bssid       = frame_types.BSSID(item[1])
            rssi        = item[3]
            
            log(LOGGING_LEVEL_VERBOSE, f"scanned={ssid}, {bssid}, {rssi}")
            
            state.get_ssid(bssid, default=ssid)
            internal_id = state.get_network_id(ssid)
            RSSIs[bssid] = RSSIs.get(bssid, 0) + rssi * weight
            

    log(LOGGING_LEVEL_INFO, f"Completed scan, found={len(state.ssid_table)} SSIDs and {len(state.bssid_table)} BSSIDs")
    for ssid, internal_id in state.ssid_table.items():
        log(LOGGING_LEVEL_VERBOSE, f"{ssid} -> {internal_id}")

    for bssid, rssi in RSSIs.items():
        ssid        = state.get_ssid(bssid, None)
        internal_id = state.get_network_id(ssid)
        rssi        = math.trunc(rssi)
        
        if rssi < -127:
            continue 
        
        log(LOGGING_LEVEL_VERBOSE, f"Inserting RSSI for {bssid} -> '{ssid}' -> {internal_id}, with value={rssi}")
        records.append(frame_types.Record(internal_id, frame_types.RSSI(rssi)))
        
        
def handle_frame(frame: Frame, frame_stack: FrameStack, port):
    if frame.cmd == frame_types.Cmd_SetPosition:
        state.set_position(frame)
    
    elif frame.cmd == frame_types.Cmd_SetParams:
        state.set_params(frame)
        
    elif frame.cmd == frame_types.Cmd_RequestRetransmit:
        proc_rx_request_retransmit(frame, frame_stack, port)
        
    elif frame.cmd == frame_types.Cmd_RequestAck:
        proc_rx_request_ack(frame_stack, port)
        
    elif frame.cmd == frame_types.Cmd_Ack:
        proc_rx_ack(frame, frame_stack)
        
    elif frame.cmd == frame_types.Cmd_EndOfTransmission:
        state.active_connection = False
        log(LOGGING_LEVEL_INFO, "Close down communication.")
        
        
    

def decode_frame_queue(frameStack: FrameStack, port: machine.UART):
    log(LOGGING_LEVEL_DEBUG, "Decoding frame from queue...")
    code, result, consumed, rx_frame_id = frame_ops.rx_frame_blocking(port)

    if code == Result_Err:
        log(LOGGING_LEVEL_ERROR, f"Parsing error ={result}")
        response = frame_types.frame_from_cmd(frame_types.Cmd_RequestRetransmit, rx_frame_id, {})
        frame_ops.tx_frame_blocking(response, frameStack, port)
        return
    
    handle_frame(result, frame_stack, port)


def init():
    global state
    global frame_stack
    global cam_frame_stack
    
    nic.active(True)
    
    UART2.init(115200, bits=8, parity=None, stop=1, tx=machine.Pin(UART2_TX_PIN), rx=machine.Pin(UART2_RX_PIN), timeout=1000)
    log(LOGGING_LEVEL_DEBUG, "Serial Inteface initialized")
    
    state = State(onPositionChange)
    frame_stack = FrameStack()
    cam_frame_stack = FrameStack()
    
    procs.proc_rx_handshake(UART2, frame_stack)
    log(LOGGING_LEVEL_INFO, "Handshake performed successfully")
    
    
    def onPrepCall(params):
        state.set_params(params)
    
    procs.proc_rx_reset(UART2, frame_stack, onPrepCall)
    log(LOGGING_LEVEL_INFO, "Reset performed successfully")
    
    state.active_connection = True
    

def main_loop():
    global frame_stack
    global state
    
    while state.active_connection:
        
        # Polling
        while UART2.any():
            decode_frame_queue(frame_stack, UART2)
            
        # Acting
        measure_rssi(state)
        procs.proc_tx_record_transmission(UART2, frame_stack, state)
        
        advance_step(state, frame_stack, UART2)
        

def main():
    while True:
        init()
        
        try:
            main_loop()
        except Exception as e:
            log(LOGGING_LEVEL_ERROR, "Unexpected error on main loop...")
            frame_ops.tx_new_frame(frame_types.Cmd_EndOfTransmission, {}, frame_stack, UART2)
            state.active_connection = False
            raise e


if __name__ == "__main__":
    main()