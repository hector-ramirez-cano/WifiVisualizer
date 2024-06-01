LOGGING_LEVEL_ALL = 0
LOGGING_LEVEL_VERBOSE = 1
LOGGING_LEVEL_DEBUG = 2
LOGGING_LEVEL_INFO = 3
LOGGING_LEVEL_WARNING = 4
LOGGING_LEVEL_ERROR = 5
LOGGING_LEVEL_NONE = 6

LABELS = (
    "[ALL  ]",
    "[VERB ]",
    "[DEBUG]",
    "[INFO ]",
    "[WARN ]",
    "[ERROR]",
)

logging_enabled = True
logging_level   = LOGGING_LEVEL_DEBUG
outputFn        = print

unflushed_logs = []

def log(level: int, msg: str):
    global logging_level

    unflushed_logs.append({"severity": level, "msg": "[ESP32-Cam]" + msg})

    if level >= logging_level:
        msg = LABELS[level] + msg
        outputFn(msg)
