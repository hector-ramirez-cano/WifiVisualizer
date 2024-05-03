from logging import *

import frame_types
import direction



class State:
    def __init__(self, onPositionChange):
        self.position              = frame_types.Position(0, 0)
        self.step_size             = frame_types.StepSize(0, 0)
        self.direction             = direction.Direction()
        self.measurements_per_step = 0
        self.known_position        = False
        self.active_connection     = False
        self.ssid_table            = dict() # ssid  -> internal_id
        self.bssid_table           = dict() # bssid -> ssid
        self._curr_new_net_id      = 0
        self.pending_ssids         = list()
        self.pending_bssids        = list()
        self.pending_records       = list()
        self.onPositionChange      = onPositionChange
        
        
    def get_network_id(self, ssid: SSID):
        if ssid in self.ssid_table:
            return self.ssid_table[ssid]
        
        self._curr_new_net_id += 1
        self.ssid_table[ssid] = frame_types.NetworkId(self._curr_new_net_id)
        self.pending_ssids.append(ssid)
        
        log(LOGGING_LEVEL_VERBOSE, f"Found new SSID '{ssid}' -> {self._curr_new_net_id}")
        
        return self.ssid_table[ssid]
    
    def get_ssid(self, bssid: BSSID, default=None):
        if bssid in self.bssid_table:
            return self.bssid_table[bssid]
        
        self.pending_bssids.append(bssid)
        self.bssid_table[bssid] = default
        log(LOGGING_LEVEL_VERBOSE, f"Found new BSSID '{bssid}' -> {default}")
        
        


    def set_params(self, frame: frame_types.Frame):
        """
            Extrae información de un Frame SetParams
            Espera que la validación del tipo de cmd se haga por quien llame la función
        
            :param frame: frame del que se va a extraer la configuración
        """
        old_pitch = self.position.pitch
        old_yaw   = self.position.yaw
        
        self.position.pitch        = frame.fields["position" ].pitch
        self.position.yaw          = frame.fields["position" ].yaw
        self.step_size.pitch_step  = frame.fields["step_size"].pitch_step
        self.step_size.yaw_step    = frame.fields["step_size"].yaw_step
        self.measurements_per_step = frame.fields["measurements_per_step"]
        
        log(LOGGING_LEVEL_VERBOSE, f"pitch_step={self.step_size.pitch_step}")
        log(LOGGING_LEVEL_VERBOSE, f"yaw_step={self.step_size.yaw_step}")
        
        self.onPositionChange(self, old_pitch, old_yaw)
        
        
    def set_position(self, frame: frame_types.Frame):
        """
            Extrae información deun Frame SetPosition
            Espera que la validación del tipo de cmd se haga por quien llame la función
            
            :param frame: frame del que se va a extraer la configuración
        """
        old_pitch = self.position.pitch
        old_yaw   = self.position.yaw
        
        self.position.pitch = frame.fields["position" ].pitch
        self.position.yaw   = frame.fields["position" ].yaw
        self.onPositionChange(self, old_pitch, old_yaw)
        
        
    def set_position(self, new_pitch, new_yaw):
        old_pitch = self.position.pitch
        old_yaw   = self.position.yaw
        
        self.position.pitch = new_pitch
        self.position.yaw   = new_yaw
        
        self.onPositionChange(self, old_pitch, old_yaw)
