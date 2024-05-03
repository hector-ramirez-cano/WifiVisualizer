import machine
import webrepl
import network
import config
import time


from logging import *

machine.Pin(33, machine.Pin.OUT).on()

ssid     = config.wifi_config['ssid']
password = config.wifi_config['password']
wlan = network.WLAN(network.STA_IF)

def wifi_conecta():
    global wlan
    time.sleep_ms(500)
    
    wlan.active(True) #Activa el Wifi
    wlan.scan()
    wlan.config(dhcp_hostname='esp32cam')
    
    start_time = time.ticks_ms()
    delta = 0
    timeout_ms = config.wifi_config.get("timeout_ms", 10_000)
    
    log(LOGGING_LEVEL_INFO, f"Intentando conectar con ssid={ssid}, timeout_ms={timeout_ms}")
    while not wlan.isconnected() and delta < timeout_ms:
        try:
            wlan.connect(ssid, password) # Hace la conexión
        except:
            pass
        delta = time.ticks_ms() - start_time
        time.sleep(0.5)
        
    if not wlan.isconnected():
        log(LOGGING_LEVEL_ERROR, f"Fallo en obtener conexión ssid={ssid} pass={password}")
        return
        
    log(LOGGING_LEVEL_INFO, 'Conexion con el WiFi %s establecida' % ssid)
    log(LOGGING_LEVEL_INFO, str(wlan.ifconfig())) #Muestra la IP y otros datos del Wi-Fi

wifi_conecta()
webrepl.start()

machine.Pin(33, machine.Pin.OUT).off()
