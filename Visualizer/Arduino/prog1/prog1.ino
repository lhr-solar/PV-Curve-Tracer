/**
 * This program is used to emulate the STM32 Nucleo on an Arduino UNO. For purposes of testing the Visualizer and Command application.
 */
#include <Arduino.h>
void setup() {
   Serial.begin(9600);
   while (!Serial); // wait for serial port to connect. Needed for native USB
}
int iter = 0;
void loop() {
   delay(10000); // delay 10 seconds
   Serial.print("Curve Tracer Log V0.1.0. Authored by Matthew Yu. This file is property of UTSVT, 2020.");
}
