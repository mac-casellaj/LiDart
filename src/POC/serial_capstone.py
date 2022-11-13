#https://docs.m2stud.io/ee/arduino/4-Serial-Communication/
import serial
import struct
import time

data = [127, 200];
ser = serial.Serial("COM3", timeout=1, baudrate=9600)
print(ser.name)
time.sleep(0.1) # arduino will be reset when serial port is opened, wait it to boot

def calc_checksum(data):
    calculated_checksum = 0
    for byte in data:
        calculated_checksum ^= byte
    return calculated_checksum

def read_packet():

    # check start sequence
    if ser.read() != b'\x10':
        return None

    if ser.read() != b'\x02':
        return None

    payload_len = ser.read()[0]
    if payload_len != 2:
        return None # payload length error
    payload = ser.read(payload_len)

    checksum = ser.read()[0]
    if checksum != calc_checksum(payload):
        return None # checksum error

    # check end sequence
    if ser.read() != b'\x10':
        return None
    if ser.read() != b'\x03':
        return None    

    # valid packet received
    return payload

def send_packet():
    tx = b'\x10\x02' # start sequence
    tx += struct.pack("<B", 2) # length of data
    packed_data = struct.pack("<BB", *data)
    tx += packed_data
    tx += struct.pack("<B", calc_checksum(packed_data))
    tx += b'\x10\x03' # end sequence
    print("Sending:", tx.hex())
    ser.write(tx)

def main():
    send_packet()
    time.sleep(0.5)
    payload = read_packet()
    unpacked = struct.unpack("<BB", payload)
    print("Data recevied", unpacked[:2])
    print("\n")

main()