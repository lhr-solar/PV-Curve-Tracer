/**
 * @file main.cpp
 * @author Matthew Yu (matthewjkyu@gmail.com)
 * @brief Testing program for the IV Curve Tracer.
 * @version 0.1
 * @date 2021-09-23
 * @copyright Copyright (c) 2021
 * @note
 * Modify __DEBUG_TUNING__ to true to switch to manual calibration 
 * mode. Modify the controller sections to optimize resolution and
 * breadth of the sampling scheme. Serial baud rate is 19200 bits 
 * per second.
 */

#include "mbed.h"

const bool __DEBUG_TUNING__ = false;

// 19200 baud rate.
#define BLINKING_RATE           250ms
#define SETTLING_TIME           15000 // us
#define ITERATIONS              5

DigitalOut ledHeartbeat(D1);
AnalogIn sensorVoltage(A6);
AnalogIn sensorCurrent(A0);
AnalogOut dacControl(A3);
CAN can(D10, D2);
CANMessage msg;

/** Tickers. */
LowPowerTicker tickHeartbeat;
void heartbeat(void) {ledHeartbeat = !ledHeartbeat;}

enum Mode {
    CELL,
    MODULE,
    ARRAY
};

float calibrateDACOut(float in) {
    // const float slope = 9.9539;
    // const float intercept = 0.0583;
    return in;// * slope + intercept;
}

float calibrateVoltageSensor(float in, float current, int numIterations, enum Mode mode) {
    // + calibration offset for internal PCB resistances causing a voltage drop prior to the voltage sensor. 
    switch (mode) {
        case CELL:
            return 1.1047 * in / numIterations;
        case MODULE:
            return 5.4591 * in / numIterations;
        case ARRAY:
            return 111.8247 * in / numIterations;
        default:
            tickHeartbeat.detach();
            ledHeartbeat = 1;
            while (1) {}
    }
}

float calibrateCurrentSensor(float in, int numIterations) {
    return 8.1169 * in / numIterations;
}

int main() {
    tickHeartbeat.attach(&heartbeat, 500ms);
    dacControl = 0.0; // 1.0 for open circuit, 0.0 for short circuit
    enum Mode mode = MODULE;

    if (__DEBUG_TUNING__) {
        printf("DEBUG MODE\n");
        ThisThread::sleep_for(5000ms);

        while(1) {
            ThisThread::sleep_for(1000ms);
            float sVolt = 0.0;
            float sCurr = 0.0;

            for (uint8_t j = 0; j < ITERATIONS; ++j) {
                wait_us(SETTLING_TIME);
                sVolt += sensorVoltage.read();
                sCurr += sensorCurrent.read();
            }

            float dacVolt = calibrateDACOut(dacControl);
            sCurr = calibrateCurrentSensor(sCurr, ITERATIONS);
            sVolt = calibrateVoltageSensor(sVolt, sCurr, ITERATIONS, mode);
            printf(
                // "%f,%f,%f,%f\n", 
                "Open Circuit\nGate (V): %f, VSense (V): %f, ISense (A): %f, V*I (W): %f\n", 
                dacVolt, 
                sVolt,
                sCurr,
                sVolt * sCurr
            );
        }
    } else {
        printf("SCAN MODE\n");
        printf("\n\nGate (V),Voltage (V),Current (A)\n");
        ThisThread::sleep_for(10000ms);

        bool forward = true;
        while (1) {

            // [0.325, 0.4, 0.00025]: 25 iterations at 1 mS each.
            if (forward) {
                for (float i = 0.25; i <= 0.5; i += 0.001) {
                    dacControl = i;
                    float sVolt = 0.0;
                    float sCurr = 0.0;

                    for (uint8_t j = 0; j < ITERATIONS; ++j) {
                        wait_us(SETTLING_TIME);
                        sVolt += sensorVoltage.read();
                        sCurr += sensorCurrent.read();
                    }

                    float dacVolt = calibrateDACOut(dacControl);
                    sCurr = calibrateCurrentSensor(sCurr, ITERATIONS);
                    sVolt = calibrateVoltageSensor(sVolt, sCurr, ITERATIONS, mode);
                    printf(
                        "%f,%f,%f,%f\n", 
                        dacVolt, 
                        sVolt,
                        sCurr,
                        sVolt * sCurr
                    );
                }
                forward = false;
            } else {
                for (float i = 0.5; i >= 0.25; i -= 0.001) {
                    dacControl = i;
                    float sVolt = 0.0;
                    float sCurr = 0.0;

                    /* Capture the average of ITERATIONS reads. */
                    for (uint8_t j = 0; j < ITERATIONS; ++j) {
                        wait_us(SETTLING_TIME);
                        sVolt += sensorVoltage.read();
                        sCurr += sensorCurrent.read();
                    }

                    float dacVolt = calibrateDACOut(dacControl);
                    sCurr = calibrateCurrentSensor(sCurr, ITERATIONS);
                    sVolt = calibrateVoltageSensor(sVolt, sCurr, ITERATIONS, mode);
                    printf(
                        "%f,%f,%f,%f\n", 
                        dacVolt, 
                        sVolt,
                        sCurr,
                        sVolt * sCurr
                    );
                }
                forward = true;
            }
        }
    }
}


