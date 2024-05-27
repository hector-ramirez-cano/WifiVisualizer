import math

def map(x, in_min, in_max, out_min, out_max):
    """
        Transforma un valor de un rango a otro
        ver: https://www.arduino.cc/reference/en/language/functions/math/map/
    """
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min

def angle_from_gravity(acc):
    """
        Calcula la dirección en que el acelerómetro está mirando, dada una medición de gravedad
        
        + ─ ─ +
        │  ┌──│ ─> d
        │  │θ │
        + ─ ─ + 
        g_x │
            v
        
        considerando que:
            g es la aceleración de gravedad, ║g║ = 9.81 m/s²
            x es la aceleración adicional
            d es la dirección unitaria en que apunta
            θ es el ángulo entre d y g_x
            acc es el g_x • d 
        
        el vector leído por el acelerómetro será g_x = ║g║ ║x║
        Si consideramos que la única fuerza ejercida sobre el acelerómetro es la gravedad, entonces:
            ║g_x║ = ║g║ ║1║ = ║g║
            
        considerando que:
                g_x • d   = ║g_x║ ║d║ cos θ
                acc       = ║g║       cos θ
                acc / ║g║ =           cos θ
            acos(acc / ║g║)=               θ
            
        :param acc: aceleración medida por el MPU6050
        :return   : dirección del eje en deg respecto de la horizontal
    """
    # para garantizar que x = 1, limitamos el rango de acc / 9.81 a [-1, 1]
    acc_g =  max(-1, min(acc/9.81, 1))
    
    theta = math.acos(acc_g)
    theta = math.degrees(theta)
    return 90 - theta

def deg_clamp(val):
    if val > 360:
        val = val - (val // 360)*360
    elif val < 0:
        val = 360 + val + (abs(val) // 360)*360
    
    return val


def clamp(val, lower, upper):
    if val < lower:
        val = lower
    elif val > upper:
        val = upper
    
    return val