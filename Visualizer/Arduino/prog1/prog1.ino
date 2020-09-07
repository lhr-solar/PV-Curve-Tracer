/**
 * This program is used to emulate the STM32 Nucleo on an Arduino UNO. For purposes of testing the Visualizer and Command application.
 * If you get the flash error: "can't open device "/dev/tty/ACM0": Permission denied", run the following command: `sudo chmod a+rw /dev/ttyACM0`.
 */
#include <Arduino.h>
#define MAX_TESTS_BUFFER 20
// function signatures
void execute_command(char* command);
void receive_data(char* buffer);
void send_data(char* buffer);
bool is_buffer_full(int buff_ptr_R, int buff_ptr_W);
bool is_buffer_empty(int buff_ptr_R, int buff_ptr_W);
bool read_command(char* buffer, char* command);

// globals and constants
const int BUFFER_SIZE = 100;
char buffer[BUFFER_SIZE];
int buff_ptr_R = 0;
int buff_ptr_W = 0;
String header = "Curve Tracer Log V0.1.0. Authored by Matthew Yu. This file is property of UTSVT, 2020.";
bool start_flag = false;

// begins waiting for serial.
void setup() {
   Serial.begin(28800);
   while (!Serial); // wait for serial port to connect. Needed for native USB
//   Serial.println("[ARDUINO] Starting up.");
//   delay(5000);
}

// main program loop
void loop() {
  // look for updates from the buffer
  receive_data(buffer);

//  static int frame = 0;
  // these println are for serial debugging. remove after testing.
//  char buff[BUFFER_SIZE];
//  sprintf (buff, "[ARDUINO] Iteration %i| WPTR %i| RPTR %i| Received %s", frame, buff_ptr_W, buff_ptr_R, buffer);
//  Serial.println(buff);
//  frame++;

  // attempt to transcribe any data in the buffer
  char command[BUFFER_SIZE];
  if (read_command(buffer, command)) {
//    sprintf (buff, "[ARDUINO] Command: %s", command);
//    Serial.println(buff);

    // parse and execute the command if we got something valid
    execute_command(command);
    // clear command
    memset(command, 0, sizeof(command));
  }
}

typedef struct Test {
  int test_id;
  float v_start;
  float v_end;
  float v_res;
} Test;
Test tests[MAX_TESTS_BUFFER] = {{0, 0.0, 0.0, 0.0}}; // 50 commands we can hold at a single time, initialized to 0
int test_idx = -1;
/**
 * execute_command attempts to run an execution regime and send it back via serial. Potentially Non blocking.
 *  char* buffer - reference to the command to execute.
 */
void execute_command(char* command) {
  char start_command[] = "START";
  char test_command[] = "TEST";

  // throw out the command identifier for further parsing.
  strtok(command, " ");
  
  // check for TEST command to set parameters
  bool found = true;
  for (int i = 0; i < sizeof(test_command)-1; i++) { // -1 to ignore the '\0'
    if (test_command[i] != command[i]) {
      found = false;
    }
  }
  if (found) {
    test_idx = (test_idx+1)%MAX_TESTS_BUFFER; // loop around once we're out of space
    // parse the command to insert the test here
    tests[test_idx] = Test {
      atoi(strtok(0, " ")), // test_id
      atof(strtok(0, " ")), // v_start
      atof(strtok(0, " ")), // v_end
      atof(strtok(0, " "))  // v_res
    };
//    Serial.println("[ARDUINO] Test command has been found. Waiting for Start command.");
    return;
  }
  
  // check for START command. If the start command ID matches the current TEST command ID, begin execution
  found = true;
  for (int i = 0; i < sizeof(start_command)-1; i++) { // -1 to ignore the '\0'
    if (start_command[i] != command[i]) {
      found = false;
    }
  }
  // extract ID from START command
  if (found) {
    if (atoi(strtok(0, " ")) == tests[test_idx].test_id) {
//      Serial.println("[ARDUINO] Start command has been found. Initiating test execution.");
      // TODO: fake test execution here
      int subid = 0;
      for (float i = tests[test_idx].v_start; i <= tests[test_idx].v_end; i += tests[test_idx].v_res) {
        // TODO: generate DAC out to MOSFET
        // TODO: collect V_ADC
        // format and send
        char buff[BUFFER_SIZE];

        // voltage
        float adc_read = ((float) i)/1000.0;
        float tmpVal = (adc_read < 0) ? -adc_read : adc_read;
        int data_int = tmpVal;                  // Get the integer (0).
        float tmpFrac = tmpVal - data_int;      // Get fraction (0.672).
        int data_frac = trunc(tmpFrac * 10000); // Turn into integer (672).
        if (adc_read < 0) {
          sprintf (buff, "DATA %i %i %i -%d.%04d;", tests[test_idx].test_id, subid, 0, data_int, data_frac); // last four values are faked
        } else {
          sprintf (buff, "DATA %i %i %i %d.%04d;", tests[test_idx].test_id, subid, 0, data_int, data_frac); // last four values are faked
        }
        send_data(buff);

        // current
        adc_read = 6.672;
        tmpVal = (adc_read < 0) ? -adc_read : adc_read;
        data_int = tmpVal;                    // Get the integer (0).
        tmpFrac = tmpVal - data_int;          // Get fraction (0.672).
        data_frac = trunc(tmpFrac * 10000);   // Turn into integer (672).
        if (adc_read < 0) {
          sprintf (buff, "DATA %i %i %i -%d.%04d;", tests[test_idx].test_id, subid, 1, data_int, data_frac); // last four values are faked
        } else {
          sprintf (buff, "DATA %i %i %i %d.%04d;", tests[test_idx].test_id, subid, 1, data_int, data_frac); // last four values are faked
        }
        send_data(buff);

        subid ++;
      }
      char buff[BUFFER_SIZE];
      sprintf (buff, "END %i;", tests[test_idx]);
      send_data(buff);
      return;  
    }
  }

//  Serial.println("[ARDUINO] Not a valid command.");
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
    int val = Serial.read();
    // ignore string endings
    if (val != '\0') {
      buffer[buff_ptr_W] = (char) val;      
    }
    buff_ptr_W = (buff_ptr_W+1)%BUFFER_SIZE;
  }
}

/**
 * send_data looks to send data across serial to the computer.
 * Args:
 *  String buffer - string to send over serial.
 */
void send_data(char* buffer) {
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
bool read_command(char* buffer, char* command) {
  // start with the read pointer, look for a byte with the value 59 (';').
  bool found = false;
  int total = 0;
  int curr_idx = buff_ptr_R;

  // read until we can't or when we found a command
  while (!is_buffer_empty(curr_idx, buff_ptr_W) && !found) {
    if (buffer[curr_idx] == ';') {
      found = true;
    }
    curr_idx = (curr_idx+1)%BUFFER_SIZE;
    total++;
  }

  // if we found it, fill up the command string pointer
  if (found) {
    int idx = buff_ptr_R;
    for (int i = 0; i < total; i++) {
      command[i] = buffer[idx];
      idx = (idx + 1)%BUFFER_SIZE;
    }
    // append null
    command[total] = '\0';
    
    // update the read pointer
    buff_ptr_R = curr_idx;
  }

  return found;
}
