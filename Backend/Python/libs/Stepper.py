import time
import machine

# only test for uln2003
class Stepper:
    FULL_ROTATION = int(4075.7728395061727 / 8) # http://www.jangeox.be/2013/10/stepper-motor-28byj-48_25.html

    HALF_STEP = [
        [0, 0, 0, 1],
        [0, 0, 1, 1],
        [0, 0, 1, 0],
        [0, 1, 1, 0],
        [0, 1, 0, 0],
        [1, 1, 0, 0],
        [1, 0, 0, 0],
        [1, 0, 0, 1],
    ]

    FULL_STEP = [
        [1, 0, 1, 0],
        [0, 1, 1, 0],
        [0, 1, 0, 1],
        [1, 0, 0, 1]
    ]
    def __init__(self, mode, pin1, pin2, pin3, pin4, delay):
        if mode=='FULL_STEP':
            self.mode = self.FULL_STEP
        else:
            self.mode = self.HALF_STEP
        self.pin1 = machine.Pin(pin1, machine.Pin.OUT)
        self.pin2 = machine.Pin(pin2, machine.Pin.OUT)
        self.pin3 = machine.Pin(pin3, machine.Pin.OUT)
        self.pin4 = machine.Pin(pin4, machine.Pin.OUT)
        self.delay = delay  # Recommend 10+ for FULL_STEP, 1 is OK for HALF_STEP
        
        # Initialize all to 0
        self.reset()
        
    def step(self, count, direction=1):
        """Rotate count steps. direction = -1 means backwards"""
        if count<0:
            direction = -1
            count = -count
        for x in range(count):
            for bit in self.mode[::direction]:
                self.pin1(bit[0])
                self.pin2(bit[1])
                self.pin3(bit[2])
                self.pin4(bit[3])
                time.sleep_ms(self.delay)
        self.reset()
    def angle(self, r, direction=1):
        if r < 0:
            direction = -1
            r = -r
        
        self.step(int(self.FULL_ROTATION * r / 360), direction)
        
    def reset(self):
        # Reset to 0, no holding, these are geared, you can't move them
        self.pin1(0) 
        self.pin2(0) 
        self.pin3(0) 
        self.pin4(0)

def create(pin1, pin2, pin3, pin4, delay=2, mode='HALF_STEP'):
    return Stepper(mode, pin1, pin2, pin3, pin4, delay)

