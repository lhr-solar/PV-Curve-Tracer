/**
 * This program is used to emulate the STM32 Nucleo on an Arduino UNO. For purposes of testing the Visualizer and Command application.
 */
#include <Arduino.h>
// function signatures
void execute_command();
void receive_data(char* buffer);
void send_data(char* buffer);
bool is_buffer_full(int buff_ptr_R, int buff_ptr_W);
bool is_buffer_empty(int buff_ptr_R, int buff_ptr_W);
bool read_command(char* buffer, String* command);

// globals and constants
const int BUFFER_SIZE = 100;
char buffer[BUFFER_SIZE];
int buff_ptr_R = 0;
int buff_ptr_W = 0;
String header = "Curve Tracer Log V0.1.0. Authored by Matthew Yu. This file is property of UTSVT, 2020.";
bool start_flag = false;

// begins waiting for serial.
void setup() {
   Serial.begin(9600);
   while (!Serial); // wait for serial port to connect. Needed for native USB
}

// main program loop
void loop() {
  // look for updates from the buffer
  receive_data(buffer);

  // attempt to transcribe any data in the buffer
  String command = "";
  if (read_command(buffer, &command)) {
    // parse the command if we got something valid
    // if valid parse, execute
    // run a second thread/pseudothread here
    execute_command();
  }
}

// some command param in here
/**
 * execute_command attempts to run an execution regime and send it back via serial. Potentially Non blocking.
 */
void execute_command() {
  // if START command, set the flag

  // if TEST command and START flag hasn't been set, ignore.
  // else do some execution
    // send fake data
    String fake_data[] = {
      "Hello World",
      "This is a test"
    };
    
    for(int i = 0; i < sizeof(fake_data)/sizeof(fake_data[0]); i++) {
      send_data(fake_data[i]);
    }
}

/**
 * receive_data looks to fill the buffer with data from serial.
 * Args:
 *  char* buffer - reference to the buffer to fill.
 */
void receive_data(char* buffer) {
  // while we can read data and can still update the buffer
  while(Serial.available() > 0 && !is_buffer_full(buff_ptr_R, buff_ptr_W)) {
    // read in the byte into the buffer and move to the next spot
    buffer[buff_ptr_W] = (char) Serial.read();
    buff_ptr_W = (buff_ptr_W+1)%BUFFER_SIZE;
  }
}

/**
 * send_data looks to send data across serial to the computer.
 * Args:
 *  String buffer - string to send over serial.
 */
void send_data(String buffer) {
   Serial.print(buffer);
}

/**
 * is_buffer_full checks to see if we can write more to the buffer.
 * Args:
 *  int buff_ptr_R - read pointer of the buffer
 *  int buff_ptr_W - write pointer of the buffer
 * Return:
 *  boolean true if full, false elsewise
 */
bool is_buffer_full(int buff_ptr_R, int buff_ptr_W) {
  if (buff_ptr_R == (buff_ptr_W+1)%BUFFER_SIZE) return true;
  else return false;
}

/**
 * is_buffer_empty checks to see if we can read more from the buffer.
 * Args:
 *  int buff_ptr_R - read pointer of the buffer
 *  int buff_ptr_W - write pointer of the buffer
 * Return:
 *  boolean true if empty, false elsewise
 */
bool is_buffer_empty(int buff_ptr_R, int buff_ptr_W) {
  if (buff_ptr_R == buff_ptr_W) return true;
  else return false;
}

/**
 * read_command attempts to read a complete command from the buffer, adjusting pointers as required.
 * Args:
 *  char* buffer - reference to the buffer to read
 *  String* command - pointer to the command string to retrieve, if possible.
 * Return:
 *  boolean true if a command has been extracted, false elsewise
 */
bool read_command(char* buffer, String* command) {
  return false;
}
