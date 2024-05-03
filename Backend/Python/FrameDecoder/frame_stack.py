import frame_types


class FrameStack:
    tx_frame_queue: list[frame_types.Frame]
    rx_frame_queue: list[frame_types.Frame]
    UNACK_THRESHOLD = 10
    
    def __init__(self):
        self.local_frame_id         = 0     # FrameID del mensaje más reciente
        self.ongoing_rx_request_ack = False # Indica si se está en el proceso de transmisión de RequestAck, para evitar recursión infinita
        self.tx_frame_queue         = []    # Cola de Frames recibidos a la espera de Ack
        self.rx_frame_queue         = []    # Cola de Frames enviados a la espera de Ack

        self.remote_ackd_frame_id   = 0
