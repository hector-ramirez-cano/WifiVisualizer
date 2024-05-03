import machine
import utils
import mpu6050

# Clase base
class sensor():
    def __init__(self, pin : machine.Pin):
        return
    
    def measure(self):
        raise "[ERROR]El método 'Measure' no está definido en la clase"
    
class ACCEL(sensor):
    def __init__(self, sda: int, scl: int):
        i2c = machine.SoftI2C(scl=machine.Pin(scl), sda=machine.Pin(sda))
        
        self.handle = mpu6050.MPU6050(scl=scl, sda=sda)
        
        
    def measure(self):
        acc  = self.handle.read_accel_data()
        gyro = self.handle.read_gyro_data()
        
        return (acc["x"], acc["y"], acc["z"], gyro["x"], gyro["y"], gyro["z"])

