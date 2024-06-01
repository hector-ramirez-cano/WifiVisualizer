# Imports del sistema
import machine

# Imports al scope
from logging import *
from state import State
from frame_stack import FrameStack

# Imports como namespace
import frame_ops
import frame_types


def proc_rx_reset(port: machine.UART, frame_stack: FrameStack, on_prep_call, onTimeout=None):
    """
        Realiza el proceso de Reset desde la perspectiva del receptor

        :param on_prep_call: función que se llamará cuando se realicen las preparaciones, previo a enviar el Ready
        :param port: puerto UART sobre el que se realiza la transmisión
        :param frame_stack: instancia de FrameStack, contenedora de Frames pendientes de Ack'd
    """

    # rx SetParams
    params = frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_SetParams,
                                                lambda frame, fs, _: log(LOGGING_LEVEL_ERROR,
                                                    f"[ERROR]Unexpected reset seq. with frame {frame}. Ignoring such frame..."),
                                                    onTimeout=onTimeout
                                                    )

    # tx Ack
    ack = frame_types.frame_from_cmd(frame_types.Cmd_Ack, 0, {"frame_id": 0})
    frame_ops.tx_frame_blocking(ack, frame_stack, port)

    on_prep_call(params)

    # tx Ready
    ready = frame_types.frame_from_cmd(frame_types.Cmd_Ready, 1, {})
    frame_ops.tx_frame_blocking(ready, frame_stack, port)

    # rx Ready
    frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_Ready, lambda frame, fs, port: log(
        LOGGING_LEVEL_ERROR,
        f"Unexpected reset seq. with frame {frame}. Ignoring such frame..."),
        onTimeout=onTimeout
        )


def proc_tx_reset(port: machine.UART, frame_stack: FrameStack):
    """
        Realiza el proceso de Reset desde la perspectiva del receptor

        :param port: puerto UART sobre el que se realiza la transmisión
        :param frame_stack: instancia de FrameStack, contenedora de Frames pendientes de Ack'd
    """
    def on_unexpected(_1, _2, _3):
        log(LOGGING_LEVEL_ERROR, "Unexpected reset sequence")

    # tx SetParams
    frame = frame_types.frame_from_cmd(
        frame_types.Cmd_SetParams,
        0,
        {
            "position": frame_types.Position(0, 0),
            "measurements_per_step": 1
        }
    )
    frame_ops.tx_frame_blocking(frame, frame_stack, port)

    # rx Ack
    frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_Ack, on_unexpected)

    # rx ready
    frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_Ready, on_unexpected)

    # tx ready
    frame_ops.tx_new_frame(frame_types.Cmd_Ready, {}, frame_stack, port)


def proc_rx_handshake(port: machine.UART, frame_stack: FrameStack, onTimeout=None):
    """
        Realiza el proceso de Handshake

        :param port: puerto UART sobre el que se realiza la transmisión
        :param frame_stack: instancia de FrameStack, contenedora de Frames pendientes de Ack'd
    """
    log(LOGGING_LEVEL_INFO, f"Waiting for handshake...")
    # rx SOT
    frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_StartOfTransmission, onTimeout=onTimeout)

    # tx Ack
    ack = frame_types.frame_from_cmd(frame_types.Cmd_Ack, 0, {"frame_id": 0})
    frame_ops.tx_frame_blocking(ack, FrameStack(), port)

    # rx Reset
    frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_Reset,
                                        lambda frame: log(LOGGING_LEVEL_ERROR, f"Unexpected handshake seq. with frame {frame}"),
                                        onTimeout=onTimeout
                                        )

    # tx Ack
    ack = frame_types.frame_from_cmd(frame_types.Cmd_Ack, 0, {"frame_id": 0})
    frame_ops.tx_frame_blocking(ack, FrameStack(), port)


def proc_tx_handshake(port: machine.UART, frame_stack: FrameStack):
    handshake_frame_stack = FrameStack()

    # tx SoT
    frame_ops.tx_new_frame(frame_types.Cmd_StartOfTransmission, {}, handshake_frame_stack, port)

    # rx ack
    frame_ops.rx_frame_blocking_expect(handshake_frame_stack, port, frame_types.Cmd_Ack)

    # tx reset
    frame_ops.tx_new_frame(frame_types.Cmd_Reset, {}, handshake_frame_stack, port)

    # rx Ack
    frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_Ack)


def proc_tx_record_transmission(port: machine.UART, frame_stack: FrameStack, state: State):
    """
        Realiza la transmisión de Records de RSSI.
        Si existe un SSID o BSSID que no haya sido registrado previamente,
        se emite un Frame de tipo Cmd_AddSSID y Cmd_AddBSSID, respectivamente.

        :param port: puerto UART sobre el que se realiza la transmisión
        :param frame_stack: instancia de FrameStack, contenedora de Frames pendientes de Ack'd
        :param state: estado de configuraciones y variables de medición
    """

    for ssid in state.pending_ssids:
        frame_ops.tx_new_frame(
            frame_types.Cmd_AddSSID,
            {"ssid": ssid, "id": state.ssid_table[ssid]},
            frame_stack,
            port
        )
    state.pending_ssids.clear()

    for bssid in state.pending_bssids:
        frame_ops.tx_new_frame(
            frame_types.Cmd_AddBSSID,
            {"bssid": bssid, "id": state.ssid_table[state.bssid_table[bssid]]},
            frame_stack,
            port
        )
    state.pending_bssids.clear()

    frame_ops.tx_new_frame(
        frame_types.Cmd_RecordRSSI,
        {
            "position": state.position,
            "record_count": len(state.pending_records),
            "records": state.pending_records
        },
        frame_stack,
        port
    )
    state.pending_records.clear()


def proc_rx_ack(frame: frame_types.Frame, frame_stack: FrameStack):
    """
        Realiza la extracción de frames ack'd de la cola
        Espera que la validación del tipo de cmd se haga por quien llame la función
        
       :param frame: frame tipo Cmd_Ack del que se extrae la información
       :param frame_stack: instancia de FrameStack, contenedora de Frames pendientes de Ack'd
    """

    frame_id = frame.fields["frame_id"]
    unconfirmed = []
    for pending_frame in frame_stack.tx_frame_queue:
        if pending_frame.frame_id > frame_id:
            unconfirmed.append(pending_frame)

    if len(unconfirmed) > 0:
        log(LOGGING_LEVEL_DEBUG, f"Pending Ack = {unconfirmed}")

    frame_stack.tx_frame_queue = unconfirmed


def proc_rx_request_retransmit(frame: frame_types.Frame, frame_stack: FrameStack, port: machine.UART):
    """
       Realiza la retransmisión de frames perdidos
      :param frame: frame tipo Cmd_RequestRetransmit del que se extrae el frame a retransmitir
      :param frame_stack: Instancia de FrameStack, contenedora de lista de tuplas de (frame_id, frame), con todos los frames que no han sido ack'd
      :param port: Puerto UART sobre el que se llevará a cabo la transmisión del Cmd_RequestRetransmit
    """
    start = frame.fields["frame_id_start"]
    end = frame.fields["frame_id_end"]

    for pending_frame in frame_stack.tx_frame_queue:
        if start <= pending_frame.frame_id < end:
            log(LOGGING_LEVEL_DEBUG, f"Retransmit of frame with id={pending_frame.frame_id}")
            frame_ops.retx_frame_blocking(pending_frame, frame_stack, port)

    log(LOGGING_LEVEL_DEBUG, f"Finished retransmission of requested frames")


def proc_tx_request_retransmit(start: int, end: int, frame_stack: FrameStack, port: machine.UART):
    frame_ops.tx_new_frame(frame_types.Cmd_RequestRetransmit, {"frame_id_start": start, "frame_id_end":end}, frame_stack, port)


def proc_tx_request_ack(frame_stack: FrameStack, port: machine.UART):
    """
        Realiza la solicitud y espera a confirmación de frames
       :param frame_stack: instacia de FrameStack con los Frames pendientes a Ack'd
       :param port: Puerto UART sobre el que se llevará a cabo la transmisión del Cmd_RequestAck
    """
    request_frame = frame_types.frame_from_cmd(frame_types.Cmd_RequestAck, frame_stack.local_frame_id,
                                               {"frame_id": frame_stack.local_frame_id})

    def onRequestRetransmit(frame, frame_stack, port):
        if frame.cmd != frame_types.Cmd_RequestRetransmit:
            log(LOGGING_LEVEL_ERROR, f"Unexpected request ack seq. with frame {frame}")
            return

        log(LOGGING_LEVEL_DEBUG, f"Requested Retansmit")
        proc_rx_request_retransmit(frame, frame_stack, port)

        frame_ops.retx_frame_blocking(request_frame, frame_stack, port)

    frame_stack.ongoing_rx_request_ack = True

    # tx RequestAck
    frame_ops.tx_frame_blocking(request_frame, frame_stack, port)

    # rx Ack or RequestRetransmit
    response = False
    while not response:
        try:
            ack = frame_ops.rx_frame_blocking_expect(frame_stack, port, frame_types.Cmd_Ack,
                                                     onOther=onRequestRetransmit)
            response = True
        except Exception as e:
            log(LOGGING_LEVEL_ERROR, f"Exception {e} on response to RequestAck")
            raise e

    # rx Ack
    proc_rx_ack(ack, frame_stack)

    frame_stack.ongoing_rx_request_ack = False


def proc_rx_request_ack(frame_stack: FrameStack, port: machine.UART):
    rx_queue = frame_stack.rx_frame_queue
    rx_ids = []

    for frame in rx_queue:
        rx_ids.append(frame.frame_id)

    rx_ids.sort()

    new_most_recent_ack = frame_stack.remote_ackd_frame_id
    for id in rx_ids:
        if id < new_most_recent_ack:
            continue

        if id == new_most_recent_ack:
            new_most_recent_ack = new_most_recent_ack + 1
            continue

        # if we got here, we've lost some packets. We ask for them again and exit
        start = new_most_recent_ack
        end = id
        proc_tx_request_retransmit(start, end, frame_stack, port)
        return

    # if we got here, it means we can safely ack all ids up until new_most_recent_ack, as they're contiguous
    frame_ops.tx_new_frame(frame_types.Cmd_Ack, {"frame_id": new_most_recent_ack}, frame_stack, port)
    frame_stack.remote_ackd_frame_id = new_most_recent_ack