
class Direction():

    DIRECTION_UP   = DIRECTION_LEFT  =  1
    DIRECTION_DOWN = DIRECTION_RIGHT = -1

    def __init__(self):
        self.__vertical   = Direction.DIRECTION_UP
        self.__horizontal = Direction.DIRECTION_LEFT

    

    @property
    def horizontal(self)-> int:
        return self.__horizontal

    @property
    def vertical(self) -> int:
        return self.__vertical

    def set_up(self):
        self.__vertical = Direction.DIRECTION_UP

    def set_down(self):
        self.__vertical = Direction.DIRECTION_DOWN

    def set_left(self):
        self.__horizontal = Direction.DIRECTION_LEFT

    def set_right(self):
        self.__horizontal = Direction.DIRECTION_RIGHT

    def invert_horizontal(self):
        self.__horizontal = -self.__horizontal

    def invert_vertical(self):
        self.__vertical = -self.__vertical