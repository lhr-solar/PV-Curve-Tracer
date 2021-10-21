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
#define SETTLING_TIME           3ms
#define NUM_CONTROLLER_STATES   3

DigitalOut ledHeartbeat(D3);
AnalogIn sensorVoltage(A0);
AnalogIn sensorCurrent(A6);
AnalogOut dacControl(A3);
CAN can(D10, D2);

/** Tickers. */
LowPowerTicker tickHeartbeat;

void heartbeat(void) { ledHeartbeat = !ledHeartbeat;}

/** 
 * A section struct contains four parameters:
 * - The [start, stop) percentage of the gate voltage
 * - The percent resolution of the gate voltage step.
 * - The number of times each step should be repeated.
 */
struct section {
    float startPercent;
    float stopPercent;
    float resolutionPercent;
    uint8_t repetition;
};

struct section controller[NUM_CONTROLLER_STATES] {
    {   // MOSFET OFF RANGE [x, 0.35)
        .startPercent=0.0, 
        .stopPercent=0.35,
        .resolutionPercent=0.02,
        .repetition=10
    },
    {   // MOSFET TRANSITION RANGE [0.35, 0.40)
        .startPercent=0.35,
        .stopPercent=0.40,
        .resolutionPercent=0.00025, // <0.00025 will be too small to register changes on DAC. TODO: reduce gate OP AMP gain.
        .repetition=30
    },
    {   // MOSFET ON RANGE [0.40, 0.6]
        .startPercent=0.40,
        .stopPercent=0.6,
        .resolutionPercent=0.02,
        .repetition=10
    }
};

int main() {
    tickHeartbeat.attach(&heartbeat, 500ms);
    dacControl = 0.0; // Default force low.

    uint8_t controllerState = 0;
    if (__DEBUG_TUNING__) {
        int add = 0;
        while (1) {
            double dacControlAvg = 0;
            double sensorVoltageAvg = 0;
            double sensorCurrentAvg = 0;

            /* Capture output. */    
            for (uint8_t i = 0; i < 100; ++i) {
                ThisThread::sleep_for(5ms);
                dacControlAvg += dacControl.read();
                sensorVoltageAvg += sensorVoltage.read();
                sensorCurrentAvg += sensorCurrent.read();
            }
            /* Average samples. */
            dacControlAvg /= 100;
            sensorVoltageAvg /= 100;
            sensorCurrentAvg /= 100;

            printf(
                "Gate (V): %f\tVoltage (V): %f\tCurrent (A): %f\n", 
                dacControlAvg*9.9539 + 0.0583, 
                sensorVoltageAvg*1.1, // CELL CALIBRATION
                // sensorVoltageAvg*1.1*6.6-0.05, // MODULE CALIBRATION
                // sensorVoltageAvg*67, // ARRAY CALIBRATION
                (sensorCurrentAvg+0.1342)/0.1359 - 1
            );
        }
    } else {
        printf("\n\nGate (V)\tVoltage (V)\tCurrent (A)\n");
        ThisThread::sleep_for(5000ms);
        while (1) {
            /* Execute current controller state. */
            #define START_PERCENT   controller[controllerState].startPercent
            #define STOP_PERCENT    controller[controllerState].stopPercent
            #define RES_PERCENT     controller[controllerState].resolutionPercent
            #define REPETITION      controller[controllerState].repetition

            /* In place update of the DAC to the Gate MOSFET. */
            for (dacControl = START_PERCENT; 
                dacControl < STOP_PERCENT;
                dacControl = dacControl + RES_PERCENT) {

                float dacControlAvg = 0;
                float sensorVoltageAvg = 0;
                float sensorCurrentAvg = 0;
                /* Capture output. */    
                for (uint8_t i = 0; i < REPETITION; ++i) {
                    /* Allow for settling time. */
                    ThisThread::sleep_for(SETTLING_TIME);
                    dacControlAvg += dacControl.read();
                    sensorVoltageAvg += sensorVoltage.read();
                    sensorCurrentAvg += sensorCurrent.read();
                }

                dacControlAvg /= REPETITION;
                sensorVoltageAvg /= REPETITION;
                sensorCurrentAvg /= REPETITION;

                printf(
                    "%f\t%f\t%f\n", 
                    dacControlAvg*9.9539 + 0.0583, 
                    sensorVoltageAvg*1.1, // CELL CALIBRATION
                    // sensorVoltageAvg*1.1*6.6-0.05, // MODULE CALIBRATION
                    // sensorVoltageAvg*67, // ARRAY CALIBRATION
                    (sensorCurrentAvg+0.1342)/0.1359 - 1

                    
                    // dacControlAvg*3.3/3, 
                    // (sensorVoltageAvg*3.3/3-.01)*1.02, // CELL CALIBRATION
                    // sensorCurrentAvg*3.3/0.4*1.03
                );
            }

            #undef START_PERCENT
            #undef STOP_PERCENT
            #undef RES_PERCENT
            #undef REPETITION

            /* Shift to the next controller state. */
            controllerState = (controllerState + 1) % NUM_CONTROLLER_STATES;
        }
    }
}
