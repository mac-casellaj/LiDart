
/*==========================================================================
//https://docs.m2stud.io/ee/arduino/4-Serial-Communication/
//serial protocol: START (0x10, 0x02), LEN, DATA, CHECKSUM, END (0x10, 0x03)
//DATA contains 2 bytes - byte1 (motA) and byte2 (motB)
//input value 0 to 127: clockwise, 0 - 0% duty cycle, 127 - 100% duty cycle
//input value 128 to 255: counter-clockwise, 128 - 0% duty cycle, 255 - 100% duty cycle
//==========================================================================
*/
#include <Arduino.h>

//Pins
int ENA = 3; //motA PWM
int ENB = 5; //motB PWM
int IN1 = 4; //motA direction
int IN2 = 6; //motA direction
int IN3 = 7; //motB direction
int IN4 = 8; //motB direction

struct Data
{
  uint8_t pwmValA;
  uint8_t pwmValB;
};

struct Packet
{
  uint16_t start_seq; // 0x0210, 0x10 will be sent first
  uint8_t len;        // length of payload
  struct Data tx_data;
  uint8_t checksum;
  uint16_t end_seq;   // 0x0310, 0x10 will be sent first
};

struct Data rx_data; // store received data
struct Packet tx_packet; // store packet to be sent

//Calculate checksum by XOR-ing all the bytes
uint8_t calc_checksum(void *data, uint8_t len)
{
  uint8_t checksum = 0;
  uint8_t *addr;
  for(addr = (uint8_t*)data; addr < ((uint8_t*)data + len); addr++){
    checksum ^= *addr;
  }
  return checksum;
}

//Read packet from serial buffer
bool readPacket()
{
  uint8_t payload_length, checksum, rx;

  while(Serial.available() < 8){
    // not enough bytes to read
  }
  
  if( Serial.read() != 0x10){
    // first byte not DLE, not a valid packet
    return false;
  }
  
  if(Serial.read() != 0x02){
    // second byte not STX, not a valid packet
    return false;
  }

  payload_length = Serial.read(); // get length of payload
  if(payload_length == 2){
    if(Serial.readBytes((uint8_t*) &rx_data, payload_length) != payload_length){
      return false;
    }
  }else{
    return false;
  }
  
  checksum = Serial.read();
  if(calc_checksum(&rx_data, payload_length) != checksum){
    return false;
  }
  
  if(Serial.read() != 0x10){
    //byte is not DLE, not a valid packet
    return false;
  }

  if(Serial.read() != 0x03){
    // last byte not ETX, not a valid packet
    return false;
  }
  digitalWrite(LED_BUILTIN, HIGH);
  return true;
}

void send_packet(){
  tx_packet.len = sizeof(struct Data);
  tx_packet.tx_data.pwmValA = rx_data.pwmValA;
  tx_packet.tx_data.pwmValB = rx_data.pwmValB;
  tx_packet.checksum = calc_checksum(&tx_packet.tx_data, tx_packet.len);
  Serial.write((char*)&tx_packet, sizeof(tx_packet)); // send the packet
}

void setup()
{
  pinMode(IN1, OUTPUT);
  pinMode(IN2, OUTPUT);
  pinMode(IN3, OUTPUT);
  pinMode(IN4, OUTPUT);
  pinMode(ENA, OUTPUT);
  pinMode(ENB, OUTPUT);
  pinMode(LED_BUILTIN, OUTPUT);

  // init tx packet
  tx_packet.start_seq = 0x0210;
  tx_packet.end_seq = 0x0310;
  
  Serial.begin(9600);
  while(!Serial){
    // wait until Serial is ready
  }
}

void loop()
{ 
  if(readPacket()){
    // valid packet received, pack data in new packet and send it out
    send_packet();
  }
  
  if(rx_data.pwmValA <= 127) {
    digitalWrite(IN1, HIGH);
    digitalWrite(IN2, LOW);
    analogWrite(ENA, rx_data.pwmValA*2);
  }
  else {
    digitalWrite(IN1, LOW);
    digitalWrite(IN2, HIGH);
    analogWrite(ENA,(rx_data.pwmValA-128)*2); 
  }

   if(rx_data.pwmValB <= 127) {
    digitalWrite(IN3, HIGH);
    digitalWrite(IN4, LOW);
    analogWrite(ENB, rx_data.pwmValB*2);
  }
  else {
    digitalWrite(IN3, LOW);
    digitalWrite(IN4, HIGH);
    analogWrite(ENB,(rx_data.pwmValB-128)*2); 
  }
}
