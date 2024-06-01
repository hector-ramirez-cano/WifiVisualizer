import unittest

from frame_types import *


class MyTestCase(unittest.TestCase):

    def test_parse_header(self):
        buff = bytes.fromhex("10 00 33 44 55")
        self.assertEqual(parse_frame_header(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("10 00 33 44 55 66 77 88")
        self.assertEqual(parse_frame_header(buff)[1], (0x1, 0x00, 0x0008, 0x33445566))

        buff = bytes.fromhex("70 02 33 44 55 66 FF FF 77 88")
        self.assertEqual(parse_frame_header(buff)[1], (0x07, 0x02, 0x0A, 0x33445566))

    def test_frame_parse_sot(self):
        buff = bytes.fromhex("00")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("00 00 00 00 00 01 DB C1")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_StartOfTransmission, 1, dict()), 8))

        buff = bytes.fromhex("00 00 00 00 00 01 DB C1 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_StartOfTransmission, 1, dict()), 8))

    def test_frame_parse_ack(self):
        buff = bytes.fromhex("40")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("40 04 00 00 00 0A 00 00 00 05 11 18")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Ack, 10, {"frame_id": 5}), 12))

        buff = bytes.fromhex("40 04 00 00 00 0A 00 00 00 05 11 18 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Ack, 10, {"frame_id": 5}), 12))


    def test_frame_parse_reset(self):
        buff = bytes.fromhex("10")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("10 01 00 00 00 01 7B FB 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        buff = bytes.fromhex("10 00 00 00 00 01 4B C3 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Reset, 1, dict()), 8))

        buff = bytes.fromhex("10 00 00 00 00 01 4B C3 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Reset, 1, dict()), 8))

        buff = bytes.fromhex("10 00 00 00 00 01 4B C3 00 00 00 01 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Reset, 1, dict()), 8))

        buff = bytes.fromhex("10 00 00 00 00 01 4B C3 00 00 00 01 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Reset, 1, dict()), 8))

    def test_frame_parse_ready(self):
        buff = bytes.fromhex("20")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("20 00 00 00 00 01 BB C6 EB F9")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Ready, 1, dict()), 8))

        buff = bytes.fromhex("20 00 00 00 00 01 BB C6 3F D0")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Ready, 1, dict()), 8))

        buff = bytes.fromhex("20 00 00 00 00 01 BB C6 3F D0 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Ready, 1, dict()), 8))

        buff = bytes.fromhex("20 00 00 00 00 01 BB C6 EE D1 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_Ready, 1, dict()), 8))

    def test_frame_parse_request_pos(self):
        buff = bytes.fromhex("30")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("30 00 00 00 00 01 2B C4")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestPosition, 1, dict()), 8))

        buff = bytes.fromhex("30 00 00 00 00 01 2B C4")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestPosition, 1, dict()), 8))

        buff = bytes.fromhex("30 00 00 00 00 01 2B C4 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestPosition, 1, dict()), 8))

        buff = bytes.fromhex("30 00 00 00 00 01 2B C4 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestPosition, 1, dict()), 8))

    def test_frame_parse_request_retransmit(self):
        buff = bytes.fromhex("50")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("50 08 00 00 00 0A 00 00 00 05 00 00 00 05 BA 96")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestRetransmit, 10, {"frame_id_start": 5, "frame_id_end": 5}), 16))

        buff = bytes.fromhex("50 08 00 00 00 0A 00 00 00 05 00 00 00 05 BA 96 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestRetransmit, 10, {"frame_id_start": 5, "frame_id_end": 5}), 16))

        buff = bytes.fromhex("50 08 00 00 00 0A 00 00 00 05 00 00 00 05 BA 96 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestRetransmit, 10, {"frame_id_start": 5, "frame_id_end": 5}), 16))

    def test_frame_parse_request_ack(self):
        buff = bytes.fromhex("60")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("60 04 00 00 00 0A 00 00 00 05 7B 19")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestAck, 10, {"frame_id": 5}), 12))

        buff = bytes.fromhex("60 04 00 00 00 0A 00 00 00 05 7B 19 54 F3")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RequestAck, 10, {"frame_id": 5}), 12))

    def test_frame_parse_length_check(self):
        # Length = 10, not enough bytes in buffer
        buff = bytes.fromhex("70 0A 00 00")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

    def test_frame_parse_add_bssid(self):
        # no length
        buff = bytes.fromhex("80")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("80 00 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        buff = bytes.fromhex("80 03 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("80 03 00 00 00 01 01 01 FF")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 37, actual length >= 37
        buff = bytes.fromhex("80 25 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 64, actual length >= 64
        buff = bytes.fromhex("80 40 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 4, no data
        buff = bytes.fromhex("80 04 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        buff = bytes.fromhex("80 0A 00 00 00 01 00 00 00 01 AA BB CC DD EE FF F0 1F")
        # length = 4, Id = 1
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddBSSID, 1, {"id": NetworkId(1), "bssid": BSSID(bytes.fromhex("AA BB CC DD EE FF"))}) , 18))

    def test_frame_parse_add_ssid(self): 
        # no length
        buff = bytes.fromhex("70")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        # length = 0
        buff = bytes.fromhex("70 00 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 3
        buff = bytes.fromhex("70 03 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        # length = 3
        buff = bytes.fromhex("70 03 00 00 00 01 01 01 FF")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 37, actual length >= 37
        buff = bytes.fromhex("70 25 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 64, actual length >= 64
        buff = bytes.fromhex("70 40 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 4, no data
        buff = bytes.fromhex("70 04 00 00 00 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_NotEnoughBytes)

        # length = 4, Id = 1
        buff = bytes.fromhex("70 04 00 00 00 01 00 00 00 01 EC BC")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(1), "ssid": SSID("")}) , 12))

        # length = 4, Id = 5
        buff = bytes.fromhex("70 04 00 00 00 01 00 00 00 05 2F BD")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(5), "ssid": SSID("")}) , 12))

        # length = 4, Id = 251_988_481
        buff = bytes.fromhex("70 04 00 00 00 01 0F 05 0A 01 59 A9")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(251_988_481), "ssid": SSID("")}) , 12))

        # length = 5, Id = 1
        buff = bytes.fromhex("70 05 00 00 00 01 00 00 00 01 41 44 7C")
        left_ = parse_frame(buff)[1:3]
        right = (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(1), "ssid": SSID("A")}) , 13)
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(1), "ssid": SSID("A")}) , 13))

        # Notice if more bytes are given, they're ignored
        # length = 5, Id = 1
        buff = bytes.fromhex("70 05 00 00 00 01 00 00 00 01 41 44 7C D0 C5 42 ")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(1), "ssid": SSID("A")}) , 13))
        
        # length = 6, Id = 1
        buff = bytes.fromhex("70 06 00 00 00 01 00 00 00 01 41 42 94 CA")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(1), "ssid": SSID("AB")}) , 14))

        buff = bytes.fromhex("70 24 00 00 00 01 00 00 00 01 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 D7 6E")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(1), "ssid": SSID("ABABABABABABABABABABABABABABABAB")}), 44))

        buff = bytes.fromhex("70 25 00 00 00 01 00 00 00 01 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 42")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

    def test_frame_parse_record_rssi(self): 
        buff = bytes.fromhex("90 04 00 00 00 01 00 00 00 00")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        buff = bytes.fromhex("90 0B 00 00 00 01 00 00 00 00 00 00 00 00 00 00 00 00")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 11, pitch = 1, yaw = 2, record count = 0
        buff = bytes.fromhex("90 0C 00 00 00 01 00 00 00 01 00 00 00 02 00 00 00 00")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        # length = 11, pitch = FF, yaw = DD, record count = 1, internal id = EE, RSSI = -128
        buff = bytes.fromhex("90 11 00 00 00 01 00 00 00 FF 00 00 00 DD 00 00 00 01 00 00 00 EE 80")
        self.assertEqual(parse_frame(buff)[1], FrameError_ValueOutOfRange)

        # length = 11, pitch = 2, yaw = 1, record count = 1, internal id = 1, RSSI = 1
        buff = bytes.fromhex("90 11 00 00 00 01 00 00 00 01 00 00 00 02 00 00 00 01 00 00 00 01 01")
        self.assertEqual(parse_frame(buff)[1], FrameError_ValueOutOfRange)

        # length = 11, pitch = 2, yaw = 1, record count = 1, internal id = 1, RSSI = -82
        buff = bytes.fromhex("90 11 00 00 00 01 00 00 00 01 00 00 00 02 00 00 00 01 00 00 00 01 AE 5A 5F")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_RecordRSSI, 1, {"position": Position(1, 2), "record_count": 1, "records": [Record(NetworkId(1), RSSI(-82))]}), 25))

    def test_frame_parse_set_position(self):
        buff = bytes.fromhex("A0 07 00 00 00 01 00 00 00 01 00 00 00 02")
        self.assertEqual(parse_frame(buff)[1], FrameError_LengthValueOutOfRange)

        buff = bytes.fromhex("A0 08 00 00 00 01 00 00 00 01 00 00 00 02 78 A5")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_SetPosition, 1,  {"position": Position(1, 2)}), 16))


    def test_frame_parse_set_param(self):
        # pitch = 1, Yaw = 2, step = FF, measurements_per_step = 5
        buff = bytes.fromhex("B0 11 00 00 00 01 00 00 00 01 00 00 00 02 11 22 33 44 55 66 77 88 05 F5 0F")
        self.assertEqual(parse_frame(buff)[1:3], (frame_from_cmd(Cmd_SetParams, 1, {"position": Position(1, 2), "step_size": StepSize(0x11223344, 0x55667788), "measurements_per_step": 5}), 25))


    def test_frame_parse(self):
        # pitch = 1, Yaw = 2, step = FF, masurements_per_step = 5
        buff = bytes.fromhex("B0 11 00 00 00 01 00 00 00 01 00 00 00 02 11 22 33 44 55 66 77 88 05 F5 0F")
        right = (Frame(Cmd_SetParams, 1, {"position": Position(1, 2), "step_size": StepSize(0x11223344, 0x55667788), "measurements_per_step": 5}, Checksum(0xF50F)), 25)
        left = parse_frame(buff)[1:3]
        self.assertEqual(parse_frame(buff)[1:3], right)


    def test_frame_to_bytes(self):
        # Start of transmission
        buff = bytes.fromhex("00 00 00 00 00 01 DB C1")
        self.assertEqual(buff, frame_from_cmd(Cmd_StartOfTransmission, 1, dict()).as_bytes())

        # Reset
        buff = bytes.fromhex("10 00 00 00 00 01 4B C3")
        self.assertEqual(buff, frame_from_cmd(Cmd_Reset, 1, dict()).as_bytes())

        # Ready
        buff = bytes.fromhex("20 00 00 00 00 01 BB C6")
        self.assertEqual(buff, frame_from_cmd(Cmd_Ready, 1, dict()).as_bytes())

        # RequestPosition
        buff = bytes.fromhex("30 00 00 00 00 01 2B C4")
        self.assertEqual(buff, frame_from_cmd(Cmd_RequestPosition, 1, dict()).as_bytes())

        # Ack
        buff = bytes.fromhex("40 04 00 00 00 0A 00 00 00 05 11 18")
        self.assertEqual(buff, frame_from_cmd(Cmd_Ack, 10, {"frame_id": 5}).as_bytes())

        # RequestRetransmit
        buff = bytes.fromhex("50 08 00 00 00 0A 00 00 00 05 00 00 00 05 BA 96")
        self.assertEqual(buff, frame_from_cmd(Cmd_RequestRetransmit, 10, {"frame_id_start": 5, "frame_id_end":5}).as_bytes())

        # RequestAck
        buff = bytes.fromhex("60 04 00 00 00 0A 00 00 00 05 7B 19")
        self.assertEqual(buff, frame_from_cmd(Cmd_RequestAck, 10, {"frame_id": 5}).as_bytes())

        # AddSSID
        buff = bytes.fromhex("70 24 00 00 00 01 00 00 00 01 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 41 42 D7 6E")
        self.assertEqual(buff, frame_from_cmd(Cmd_AddSSID, 1, {"id": NetworkId(1), "ssid": SSID("ABABABABABABABABABABABABABABABAB")}).as_bytes())

        # AddBSSID
        buff = bytes.fromhex("80 0A 00 00 00 01 00 00 00 01 AA BB CC DD EE FF F0 1F")
        self.assertEqual(buff, frame_from_cmd(Cmd_AddBSSID, 1, {"id": NetworkId(1), "bssid": BSSID(bytes.fromhex("AA BB CC DD EE FF"))}).as_bytes())

        # RecordRSSI
        buff = bytes.fromhex("90 11 00 00 00 01 00 00 00 01 00 00 00 02 00 00 00 01 00 00 00 01 AE 5A 5F")
        self.assertEqual(buff, frame_from_cmd(Cmd_RecordRSSI, 1, {"position": Position(1, 2), "record_count": 1, "records": [Record(NetworkId(1), RSSI(-82))]}).as_bytes())

        # SetPosition
        buff = bytes.fromhex("A0 08 00 00 00 01 00 00 00 01 00 00 00 02 78 A5")
        self.assertEqual(buff, frame_from_cmd(Cmd_SetPosition, 1, {"position": Position(1, 2)}).as_bytes())

        # SetParam
        buff = bytes.fromhex("B0 11 00 00 00 01 00 00 00 01 00 00 00 02 11 22 33 44 55 66 77 88 05 F5 0F")
        self.assertEqual(buff, frame_from_cmd(Cmd_SetParams, 1, {"position": Position(1, 2), "step_size": StepSize(0x11223344, 0x55667788), "measurements_per_step": 5}).as_bytes())

        # TransmitPicture
        buff = bytes.fromhex("C0 0A DD DD DD DD FA 00 00 AF C1 00 00 1C 7B 7D 8B 60")
        self.assertEqual(buff, frame_from_cmd(Cmd_TransmitPicture, 0xDDDDDDDD, {"position": Position(0xFA0000AF, 0xC100001C), "body": json.loads("{}")}).as_bytes())

        # EndOfTransmission
        buff = bytes.fromhex("F0 00 00 00 00 01 2B D5")
        self.assertEqual(buff, frame_from_cmd(Cmd_EndOfTransmission, 1, dict()).as_bytes())


if __name__ == '__main__':
    unittest.main()

