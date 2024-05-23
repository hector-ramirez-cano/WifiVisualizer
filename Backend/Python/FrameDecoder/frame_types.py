# system imports
import json
import binascii

from logging import * 

import utils

FRAME_HEADER_SIZE = 6
CHECKSUM_SIZE = 2

FrameError_EmptyFrame = 1
FrameError_InvalidErrorCode = 2
FrameError_LengthValueOutOfRange = 3
FrameError_NotEnoughBytes = 4
FrameError_ValueOutOfRange = 5
FrameError_InvalidChecksum = 6

Result_Ok = 0
Result_Err = 1

Cmd_StartOfTransmission = 0
Cmd_Reset = 1
Cmd_Ready = 2
Cmd_RequestPosition = 3
Cmd_Ack = 4
Cmd_RequestRetransmit = 5
Cmd_RequestAck = 6
Cmd_AddSSID = 7
Cmd_AddBSSID = 8
Cmd_RecordRSSI = 9
Cmd_SetPosition = 10
Cmd_SetParams = 11
Cmd_TransmitPicture = 12
Cmd_TransmitLogs = 13
Cmd_EndOfTransmission = 15


def sign(val):
    if val < 0:
        return -1
    else:
        return 1


def twos_comp(val, bits):
    """compute the 2's complement of int value val"""
    if (val & (1 << (bits - 1))) != 0:  # if sign bit is set e.g., 8bit: 128-255
        val = val + (256 * -sign(val))
    return val


class BSSID:
    def __init__(self, buff: bytes):
        if len(buff) < 6:
            raise "Invalid length for BSSID bytes!"

        self.bytes = buff

    def __eq__(self, other):
        return self.__hash__() == other.__hash__()

    def __hash__(self):
        return hash(self.bytes)

    def __str__(self):
        return binascii.hexlify(self.bytes).decode()

    def as_bytes(self) -> bytes:
        return self.bytes


class SSID:
    def __init__(self, name: str):
        if len(name) > 32:
            raise Exception("Invalid SSID name length!")

        if isinstance(name, bytes):
            self.name = bytes.decode(name, "UTF-8", "replace")

        elif isinstance(name, str):
            self.name = name

        else:
            raise Exception("SSID should be a string")

    def __eq__(self, other):
        return self.__hash__() == other.__hash__()

    def __hash__(self):
        return hash(self.name)

    def __str__(self):
        return str(self.name, "UTF-8")

    def as_bytes(self) -> bytes:
        return str.encode(self.name, "UTF-8", "ignore")


class Checksum:
    def __init__(self, checksum: int):
        self.checksum = checksum

    def check(self, buff: bytes):
        return checksum_from_bytes(buff) == self

    def __eq__(self, other):
        return self.checksum == other.checksum

    def as_bytes(self) -> bytes:
        return int.to_bytes(self.checksum, 2, "big")


class RSSI:
    def __init__(self, strength: int):
        if strength < -127 or strength > 0:
            raise Exception("Invalid RSSI range. RSSI was = " + str(strength))

        self.strength = strength

    def __eq__(self, other):
        return self.strength == other.strength

    def __hash__(self):
        return hash(self.strength)

    def __str__(self):
        return str(self.strength)

    def as_bytes(self) -> bytes:
        twos_c = twos_comp(self.strength, 8)
        return int.to_bytes(twos_c, 1, "big")


class Position:
    def __init__(self, pitch: int, yaw: int):
        if pitch < 0 or pitch > 2 ** 32:
            raise "Pitch out of range!"

        if yaw < 0 or yaw > 2 ** 32:
            raise "Yaw out of range"

        self.pitch = pitch
        self.yaw   = yaw

    def __eq__(self, other):
        return self.pitch == other.pitch and self.yaw == other.yaw

    def as_bytes(self) -> bytes:
        pitch = int.to_bytes(self.pitch, 4, "big")
        yaw   = int.to_bytes(self.yaw, 4, "big")
        return pitch + yaw

    def pitch_as_deg(self):
        return Position.raw_as_deg(self.pitch)

    def yaw_as_deg(self):
        return Position.raw_as_deg(self.yaw)

    def raw_as_deg(val):
        return utils.map(val, 0, 2 ** 32, 0.0, 360.0)


class NetworkId:
    def __init__(self, network_id: int):
        if network_id < 0 or network_id > 2 ** 32:
            raise "Network Id out of range!"

        self.id = network_id

    def __eq__(self, other):
        return self.id == other.id

    def __hash__(self):
        return hash(self.id)

    def __str__(self):
        return str(self.id)

    def as_bytes(self) -> bytes:
        return int.to_bytes(self.id, 4, "big")


class Record:
    def __init__(self, network_id: NetworkId, rssi: RSSI):
        self.id = network_id
        self.rssi = rssi

    def __eq__(self, other):
        return self.id == other.id and self.rssi == other.rssi

    def as_bytes(self) -> bytes:
        return self.id.as_bytes() + self.rssi.as_bytes()


class StepSize:
    def __init__(self, pitch_step, yaw_step):
        if pitch_step < 0 or pitch_step > 2 ** 32:
            raise "Step size pitch out of range!"

        if yaw_step < 0 or yaw_step > 2 ** 32:
            raise "Step size yaw out of range!"

        self.pitch_step = pitch_step
        self.yaw_step = yaw_step

    def __eq__(self, other):
        return self.pitch_step == other.pitch_step and self.yaw_step == other.yaw_step

    def as_bytes(self) -> bytes:
        return int.to_bytes(self.pitch_step, 4, "big") + int.to_bytes(self.yaw_step, 4, "big")

    def pitch_as_deg(self):
        return utils.map(self.pitch_step, 0, 2 ** 32, 0.0, 360.0)

    def yaw_as_deg(self):
        return utils.map(self.yaw_step, 0, 2 ** 32, 0.0, 360.0)


class Frame:
    def __init__(self, cmd: int, frame_id: int, fields: dict, checksum: Checksum):
        self.cmd      = cmd
        self.checksum = checksum
        self.fields   = fields
        self.frame_id = frame_id

    def __eq__(self, other):
        return self.cmd == other.cmd and self.checksum == other.checksum and self.fields == other.fields

    def body_as_bytes(self) -> bytes:
        if self.cmd in [Cmd_StartOfTransmission, Cmd_Reset, Cmd_Ready, Cmd_RequestPosition, Cmd_EndOfTransmission]:
            return bytes()

        if self.cmd == Cmd_Ack:
            return int.to_bytes(self.fields["frame_id"], 4, "big")

        if self.cmd == Cmd_RequestRetransmit:
            return int.to_bytes(self.fields["frame_id_start"], 4, "big") + int.to_bytes(self.fields["frame_id_end"], 4, "big")

        if self.cmd == Cmd_RequestAck:
            return int.to_bytes(self.fields["frame_id"], 4, "big")

        if self.cmd == Cmd_AddSSID:
            return self.fields["id"].as_bytes() + self.fields["ssid"].as_bytes()

        if self.cmd == Cmd_AddBSSID:
            return self.fields["id"].as_bytes() + self.fields["bssid"].as_bytes()

        if self.cmd == Cmd_RecordRSSI:
            result = self.fields["position"].as_bytes()
            result = result + int.to_bytes(self.fields["record_count"], 4, "big")

            for record in self.fields["records"]:
                result = result + (record.as_bytes())

            return result

        if self.cmd == Cmd_SetPosition:
            return self.fields["position"].as_bytes()

        if self.cmd == Cmd_SetParams:
            position  = self.fields["position"].as_bytes()
            step_size = self.fields["step_size"].as_bytes()
            mps       = int.to_bytes(self.fields["measurements_per_step"], 1, "big")
            return position + step_size + mps

        if self.cmd == Cmd_TransmitPicture:
            position = self.fields["position"].as_bytes()
            body = str.encode(json.dumps(self.fields["body"], separators=(',', ':')), "utf-8")

            return position + body

        if self.cmd == Cmd_TransmitLogs:
            return str.encode(json.dumps(self.fields["logs"], separators=(',', ':')), "utf-8")

    def as_bytes(self) -> bytes:
        body = self.body_as_bytes()
        header = header_as_bytes(self.cmd, len(body), self.frame_id)
        checksum = self.checksum

        return header + body + checksum.as_bytes()


def header_as_bytes(cmd_nibble: int, length: int, frame_id: int) -> bytes:
    msb = cmd_nibble << 4 | ((length & 0x0F00) >> 8)
    lsb = (length & 0x00FF)

    buff = int.to_bytes(msb, 1, "big")
    buff = buff + int.to_bytes(lsb, 1, "big")
    buff = buff + int.to_bytes(frame_id, 4, "big")
    return buff


# @micropython.viper
def checksum_from_bytes(buff: bytes):
    crc16: int = 0xffff
    poly: int = 0xa001
    i: int = 0
    num = len(buff)
    while i < num:
        crc16 = buff[i] ^ crc16
        j: int = 8
        while j > 0:
            if crc16 & 0x0001 == 1:
                crc16 >>= 1
                crc16 ^= poly
            else:
                crc16 >>= 1
            j -= 1
        i += 1
    return Checksum(crc16)


def frame_from_cmd(cmd: int, frame_id: int, fields: dict) -> Frame:
    empty_checksum_frame = Frame(cmd, frame_id, fields, Checksum(0))

    body = empty_checksum_frame.body_as_bytes()
    header = header_as_bytes(cmd, len(body), frame_id)

    checksum = checksum_from_bytes(header + body)

    return Frame(cmd, frame_id, fields, checksum)


def parse_bssid(buff: bytes):
    if len(buff) < 6:
        return Result_Err, FrameError_NotEnoughBytes

    try:
        return Result_Ok, BSSID(buff[0:6])
    except:
        return Result_Err, FrameError_NotEnoughBytes


def parse_ssid(buff: bytes):
    try:
        name = buff.decode("UTF-8", "strict")
    except:
        name = "INVALID_UTF-8"

    return SSID(name)


def parse_rssi(buff: bytes):
    if len(buff) < 1:
        return Result_Err, FrameError_NotEnoughBytes

    rssi = int.from_bytes(buff[0:1], "big")
    rssi = twos_comp(rssi, 8)
    try:
        return Result_Ok, RSSI(rssi)
    except:
        return Result_Err, FrameError_ValueOutOfRange


def parse_position(buff: bytes):
    if len(buff) < 8:
        return Result_Err, FrameError_NotEnoughBytes

    pitch = int.from_bytes(buff[0:4], "big")
    yaw = int.from_bytes(buff[4:8], "big")

    try:
        return Result_Ok, Position(pitch, yaw)
    except:
        return Result_Err, FrameError_ValueOutOfRange


def parse_network_id(buff: bytes):
    if len(buff) < 4:
        return Result_Err, FrameError_NotEnoughBytes

    network_id = int.from_bytes(buff[0:4], "big")

    try:
        return Result_Ok, NetworkId(network_id)
    except Exception as e:
        return Result_Err, FrameError_ValueOutOfRange


def parse_record(buff: bytes):
    if len(buff) < 5:
        return Result_Err, FrameError_NotEnoughBytes

    network_id = parse_network_id(buff[0:4])
    rssi = parse_rssi(buff[4:5])

    if network_id[0] == Result_Err:
        return Result_Err, network_id[1]
    if rssi[0] == Result_Err:
        return Result_Err, rssi[1]

    return Result_Ok, Record(network_id[1], rssi[1])


def parse_multiple_records(count: int, buff: bytes):
    vec = []
    if len(buff) * 5 < count:
        return Result_Err, FrameError_NotEnoughBytes

    for i in range(count):
        start = i * 5
        end = start + 5
        record = parse_record(buff[start:end])

        if record[0] == Result_Err:
            return Result_Err, record[1]

        vec.append(record[1])

    return Result_Ok, vec


def parse_step_size(buff: bytes):
    if len(buff) < 8:
        return Result_Err, FrameError_NotEnoughBytes

    return Result_Ok, StepSize(int.from_bytes(buff[0:4], "big"), int.from_bytes(buff[4:8], "big"))


def parse_frame_header(buff: bytes):
    if len(buff) == 0:
        return Result_Err, FrameError_EmptyFrame
    if len(buff) < FRAME_HEADER_SIZE:
        return Result_Err, FrameError_NotEnoughBytes

    cmd_nibble = (buff[0] & 0xF0) >> 4

    length = (buff[0] & 0x0F) << 8 | buff[1]
    frame_length = length + FRAME_HEADER_SIZE + CHECKSUM_SIZE

    frame_id = int.from_bytes(buff[2:6], "big")

    return Result_Ok, (cmd_nibble, length, frame_length, frame_id)


def parse_cmd_body(cmd_nibble: int, length, buff: bytes):
    if cmd_nibble in range(0x0, 0x4) or cmd_nibble == 0xF:
        if length != 0:
            return Result_Err, FrameError_LengthValueOutOfRange
        else:
            return Result_Ok, (cmd_nibble, dict())

    elif cmd_nibble == Cmd_Ack:
        if length != 0x004:
            return Result_Err, FrameError_LengthValueOutOfRange

        return Result_Ok, (Cmd_Ack, {"frame_id": int.from_bytes(buff[0:5], "big")})

    elif cmd_nibble == Cmd_RequestRetransmit:
        if length != 0x008:
            return Result_Err, FrameError_LengthValueOutOfRange

        return Result_Ok, (Cmd_RequestRetransmit, { "frame_id_start": int.from_bytes(buff[0:4], "big"),
                                                    "frame_id_end"  : int.from_bytes(buff[4:9], "big")})

    elif cmd_nibble == Cmd_RequestAck:
        if length != 0x004:
            return Result_Err, FrameError_LengthValueOutOfRange

        return Result_Ok, (Cmd_RequestAck, {"frame_id": int.from_bytes(buff[0:5], "big")})

    elif cmd_nibble == Cmd_AddSSID:
        if length < 0x4 or length > 0x24:
            return Result_Err, FrameError_LengthValueOutOfRange

        net_id = parse_network_id(buff[0:4])
        if net_id[0] == Result_Err:
            return Result_Err, net_id[1]

        if length == 0x4:
            ssid = SSID("")
        else:
            ssid = parse_ssid(buff[4:])

        return Result_Ok, (Cmd_AddSSID, {"ssid": ssid, "id": net_id[1]})

    elif cmd_nibble == Cmd_AddBSSID:
        if length != 0xA:
            return Result_Err, FrameError_LengthValueOutOfRange

        net_id = parse_network_id(buff[0:4])
        if net_id[0] == Result_Err:
            return Result_Err, net_id[1]

        bssid = parse_bssid(buff[4:])
        if bssid[0] == Result_Err:
            return Result_Err, bssid[1]

        return Result_Ok, (Cmd_AddBSSID, {"bssid": bssid[1], "id": net_id[1]})

    elif cmd_nibble == Cmd_RecordRSSI:
        if length != 0x11:
            return Result_Err, FrameError_LengthValueOutOfRange

        count = int.from_bytes(buff[8:12], "big")

        position = parse_position(buff[0:8])
        if position[0] == Result_Err:
            return Result_Err, position[1]

        records = parse_multiple_records(count, buff[12:])
        if records[0] == Result_Err:
            return Result_Err, records[1]

        return Result_Ok, (Cmd_RecordRSSI, {"position": position[1], "record_count": count, "records": records[1]})

    elif cmd_nibble == Cmd_SetPosition:
        if length != 0x8:
            return Result_Err, FrameError_LengthValueOutOfRange

        position = parse_position(buff[0:8])
        if position[0] == Result_Err:
            return Result_Err, position[1]

        return Result_Ok, (Cmd_SetPosition, {"position": position[1]})

    elif cmd_nibble == Cmd_SetParams:
        if length != 0x011:
            return Result_Err, FrameError_LengthValueOutOfRange

        position = parse_position(buff[0:8])
        if position[0] == Result_Err:
            return Result_Err, position[1]

        step_size = parse_step_size(buff[8: 16])
        if step_size[0] == Result_Err:
            return Result_Err, step_size[1]
        
        log(LOGGING_LEVEL_VERBOSE, f"pitch_step={step_size[1].pitch_step}")
        log(LOGGING_LEVEL_VERBOSE, f"yaw_step={step_size[1].yaw_step}")

        measurements_per_step = buff[16]

        return (Result_Ok,
                (Cmd_SetParams,
                {
                    "position": position[1],
                    "step_size": step_size[1],
                    "measurements_per_step": measurements_per_step
                }))

    elif cmd_nibble == Cmd_TransmitPicture:
        if length < 0x00A:
            return Result_Err, FrameError_LengthValueOutOfRange

        position = parse_position(buff[0:8])
        if position[0] == Result_Err:
            return Result_Err, position[1]

        body = json.loads(str(buff[8:], "UTF_8"))

        return (
            Result_Ok,
            (Cmd_TransmitPicture,
            {
                "position": position[1],
                "body"    : body
            }))

    elif cmd_nibble == Cmd_TransmitLogs:
        if length < 0x002:
            return Result_Err, FrameError_LengthValueOutOfRange

        logs = json.loads(str(buff), "UTF-8")
        return (
            Result_Ok,
            ( Cmd_TransmitLogs, { "logs": logs } )
        )

    else:
        return Result_Err, FrameError_InvalidErrorCode


def parse_cmd(cmd_nibble: int, length: int, frame_length: int, buff: bytes):
    if len(buff) > 1 and length > len(buff) - FRAME_HEADER_SIZE:
        return Result_Err, FrameError_NotEnoughBytes

    start = FRAME_HEADER_SIZE
    end = frame_length - CHECKSUM_SIZE

    data = buff[start:end]

    return parse_cmd_body(cmd_nibble, length, data)


def parse_frame(buff: bytes) -> (int, Frame):
    status, value = parse_frame_header(buff)
    if status == Result_Err:
        return status, value
    (cmd_nibble, length, frame_length, frame_id) = value

    status, value = parse_cmd(cmd_nibble, length, frame_length, buff)
    if status == Result_Err:
        return status, value, 0, -1

    cmd, fields = value

    consumed = FRAME_HEADER_SIZE + length

    checksum = Checksum(int.from_bytes(buff[consumed:(consumed + CHECKSUM_SIZE)], "big"))

    if checksum.check(buff[0:consumed]):
        return Result_Ok, Frame(cmd, frame_id, fields, checksum), consumed + CHECKSUM_SIZE, frame_id
    else:
        return Result_Err, FrameError_InvalidChecksum, 0, frame_id


