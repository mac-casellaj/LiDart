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

struct Data {
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

const uint8_t START_1 = 0x10; //'A';
const uint8_t START_2 = 0x02; //'B';
const uint8_t END_1 = 0x10; // 'C';
const uint8_t END_2 = 0x03; // 'D';

struct Data rx_data; // store received data
struct Packet tx_packet; // store packet to be sent

// Calculate checksum by XOR-ing all the bytes
uint8_t calc_checksum(void *data, uint8_t len) {
  uint8_t checksum = 0;
  uint8_t *addr;
  for(addr = (uint8_t*)data; addr < ((uint8_t*)data + len); addr++){
    checksum ^= *addr;
  }
  return checksum;
}

int led_val = LOW;

// Read packet from serial buffer
bool readPacket() {
  // read bytes into the rx buffer, spinning on Serial.available() is probably a bad idea
  static uint8_t rx_buffer_used = 0;
  static uint8_t rx_buffer[8];

  while(Serial.available() && (rx_buffer_used != 8)) {
    rx_buffer[rx_buffer_used++] = Serial.read();
  }

  // not enough bytes to read
  if(rx_buffer_used != 8) {
    return false;
  }

  rx_buffer_used = 0;

  Serial.print("Got packet. ");

  if(rx_buffer[0] != START_1) {
    // first byte not DLE, not a valid packet
    Serial.println("START_1 Invalid");
    return false;
  }
  
  if(rx_buffer[1] != START_2) {
    // second byte not STX, not a valid packet
    Serial.println("START_2 Invalid");
    return false;
  }

  // get length of payload
  uint8_t payload_length = rx_buffer[2]; 
  
  if(payload_length == 2) {
    rx_data.pwmValA = rx_buffer[3];
    rx_data.pwmValB = rx_buffer[4];
  } else {
    Serial.print("Invalid payload length ");
    Serial.println(payload_length);
    return false;
  }
  
  uint8_t checksum = rx_buffer[5];
  Serial.print("Expected checksum ");
  Serial.print(checksum);
  Serial.print(", calculated checksum ");
  Serial.print(rx_buffer[3] ^ rx_buffer[4]);
  Serial.print(". ");
  if(calc_checksum(&rx_data, payload_length) != checksum) {
    Serial.println(" Checksum invalid");
    return false;
  }
  
  if(rx_buffer[6] != END_1) {
    //byte is not DLE, not a valid packet
    Serial.println("END_1 Invalid");
    return false;
  }

  if(rx_buffer[7] != END_2) {
    // last byte not ETX, not a valid packet
    Serial.println("END_2 Invalid");
    return false;
  }

  Serial.println("Success");
  
  led_val = (led_val == LOW) ? HIGH : LOW;
  digitalWrite(LED_BUILTIN, led_val);
  
  return true;
}

void send_packet(){
  tx_packet.len = sizeof(struct Data);
  tx_packet.tx_data.pwmValA = rx_data.pwmValA;
  tx_packet.tx_data.pwmValB = rx_data.pwmValB;
  tx_packet.checksum = calc_checksum(&tx_packet.tx_data, tx_packet.len);
  Serial.write((char*)&tx_packet, sizeof(tx_packet)); // send the packet
}

void setup() {
  pinMode(IN1, OUTPUT);
  pinMode(IN2, OUTPUT);
  pinMode(IN3, OUTPUT);
  pinMode(IN4, OUTPUT);
  pinMode(ENA, OUTPUT);
  pinMode(ENB, OUTPUT);
  pinMode(LED_BUILTIN, OUTPUT);

  digitalWrite(LED_BUILTIN, LOW);

//  digitalWrite(4, HIGH);
//  digitalWrite(IN2, LOW);
//  digitalWrite(IN3, LOW);
//  digitalWrite(IN4, LOW);
//  digitalWrite(3, HIGH);
//  digitalWrite(ENB, LOW);  

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
    // send_packet();
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
