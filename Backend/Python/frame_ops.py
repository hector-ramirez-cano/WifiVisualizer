# Imports del sistema
import machine

# Imports al scope
from logging import *
from frame_types import *
from frame_stack import FrameStack

# Imports como namespace
import frame_types
import procs


def rx_frame_blocking(frame_stack: FrameStack, port: machine.UART, onTimeout = None):
    """
        Bloquea el flujo de lógica hasta recibir un frame completo.
        En caso de fallo, retorna un error
        
        :param frame_stack: estado actual del stack de control de tramas
        :param port: Puerto UART inicializado que será utilizado para la recepción
        :return: (Result.Ok | Result.Err, ErrCode | frame, 0 | consumed_bytes, -1 | rx_frame_id)
    """
    head = None
    
    # Read the header
    while head == None:
        head = port.read(FRAME_HEADER_SIZE)

        if head == None and onTimeout is not None:
            onTimeout()
    
    # we parse the header to see how many bytes we need to read
    (result, header) = frame_types.parse_frame_header(head)
    
    # log(LOGGING_LEVEL_DEBUG, f"header = {result, header}")
    if result == Result_Err:
        return Result_Err, header, 0, -1
    
    (_, length, _, _) = header
    
    body = None
    while body == None:
        body = port.read(length+CHECKSUM_SIZE)

    frame = frame_types.parse_frame(head+body)

    if frame[0] == Result_Err:
        return frame

    frame_stack.rx_frame_queue.append(frame)

    return frame


def rx_frame_blocking_expect(frame_stack: FrameStack, port: machine.UART, expected_cmd, onOther=None, onTimeout=None) -> Frame:
    """
        Bloquea el flujo de lógica a través de rx_frame_blocking, pero
        continúa en bucle infinito hasta recibir el cmd esperado. Descarta todos los demás frames.
        
        :param frame_stack: estado actual del stack de control de tramas
        :param port: Puerto UART en que se va a dar la conexión
        :param expected_cmd: código de comando esperado
        :param onOther: función que recibe Frame que será llamada cuando se reciba un frame diferente al esperado
        :return: frame
    """
    while True:
        result = rx_frame_blocking(frame_stack, port, onTimeout=onTimeout)
        if result[0] == Result_Err:
            continue
        frame = result[1]
        
        if frame.cmd != expected_cmd:
            if onOther is not None:
                log(LOGGING_LEVEL_DEBUG , f"OnOther Called for {frame}")
                onOther(frame, frame_stack, port)
            continue
        
        return frame


def tx_new_frame(cmd: int, fields: dict, frame_stack: FrameStack, port: machine.UART):
    """
        Crea y transmite un nuevo frame con el cmd y fields dados.
        Si detecta que un RequestAck es necesario, espera hasta recibirlo.
        
        :param cmd: Tipo de comando del frame
        :param fields: diccionario de datos del cuerpo del frame
        :param frame_stack: stack de control de frames
        :param port: puerto UART por el que se llevará a cabo la transmisión
    """
    frame = frame_from_cmd(cmd, frame_stack.local_frame_id, fields)
    tx_frame_blocking(frame, frame_stack, port)
    

def tx_frame_blocking(frame: Frame, frame_stack: FrameStack, port: machine.UART):
    """
        Transmite un frame con el cmd y fields dados.
        Si detecta que un RequestAck es necesario, espera hasta recibirlo.
        
        :param cmd: Tipo de comando del frame
        :param fields: diccionario de datos del cuerpo del frame
        :param frame_stack: stack de control de frames
        :param port: puerto UART por el que se llevará a cabo la transmisión
    """
    
    tx = frame.as_bytes()
    frame_stack.tx_frame_queue.append(frame)

    port.write(tx)
    
    frame_stack.local_frame_id += 1
    
    if len(frame_stack.tx_frame_queue) >= FrameStack.UNACK_THRESHOLD and not frame_stack.ongoing_rx_request_ack:
        log(LOGGING_LEVEL_INFO, "Threshold for un-ack'd frames reached. Sending a RequestAck...")
        procs.proc_tx_request_ack(frame_stack, port)


def retx_frame_blocking(frame: Frame, frame_stack: FrameStack, port: machine.UART):
    """
        retransmite un frame con el cmd y fields dados.
        
        :param frame: Frame a retransmitir
        :param cmd: Tipo de comando del frame
        :param fields: diccionario de datos del cuerpo del frame
        :param frame_stack: stack de control de frames
        :param port: puerto UART por el que se llevará a cabo la transmisión
    """
    
    tx = frame.as_bytes()
    port.write(tx)

