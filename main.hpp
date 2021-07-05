/**
 * Project: PV Curve Tracer Board
 * File: main.hpp
 * Author: Matthew Yu (2021).
 * Organization: UT Solar Vehicles Team
 * Created on: 05/29/21
 * Last Modified: 06/29/21
 * File Description: Header file for the PV Curve Tracer board, used 
 * for testing.
 * 
 * L432KC Pinout:
 * https://os.mbed.com/media/uploads/bcostm/nucleo_l432kc_2017_10_09.png
 * Note: The following pins must be reserved during STLink debugging:
 * - PA11 | D10 | USP_DM
 * - PA12 | D2  | USB_DP
 * - PA13 | N/A | USB_NOE
 * - PC14 | D7  | RCC_OSC32_IN
 * - PC15 | D8  | RCC_OSC32_OUT
 * - PA14 | N/A | SYS_JTCK_SWCLK
 * - PA15 | N/A | SYS_JTDI
 * - PB3  | D13 | SYS_JTDO_SW0
 * - PA13 | N/A | SYS_JTMS_SWDIO
 * - PB4  | D12 | SYS_JTRST
 * - PB7  | D4  | SYS_PVD_IN
 * - PA0  | A0  | SYS_WKUP1
 * - PA2  | A7  | SYS_WKUP4
 * L432KC specific.
 */
 

/** Includes. */
#include "mbed.h"
#include <chrono>
#include <cstdio>
#include <Misc/Errors.hpp>
#include <Misc/ComIds.hpp>
#include <Fifo/Fifo.hpp>
#include <Misc/MType.hpp>
#include "VoltageAdcSensor.hpp"
#include "CurrentAdcSensor.hpp"

/** Defines. */
#define USB_TX USBTX /* A7. */
#define USB_RX USBRX /* A2. */
// #define UART2_TX D5
// #define UART2_RX D4
#define CAN_TX D2
#define CAN_RX D9 /* Errata. Should be D10. */
#define ADC_CURRENT A6
#define ADC_VOLTAGE A0
#define DAC_CONTROL A3
#define LED_HEARTBEAT D3
#define LED_SCANNING D0
#define LED_ERROR D1
#define QUEUE_SIZE 100
#define PRELUDE 0xFF

/** Struct definitions. */
/** The profile struct is used for test execution and is generated by the user. */
struct profile {
    bool complete;              /* Whether the profile is valid and can be executed. */

    /* User defined variables. */
    enum regime {               /* Test regime. Affects the scaling of the ADC readings. */
        NO_REGIME, 
        CELL, 
        MODULE, 
        SUBARRAY, 
        RESERVED1, 
        RESERVED2, 
        RESERVED3, 
        RESERVED4
    } testRegime;
    float voltageStart;         /* Starting voltage, in V (0 - 3.3 V). */
    float voltageEnd;           /* Ending voltage, in V (0 - 3.3 V). */
    float voltageResolution;    /* Voltage resolution, in V (0 - 1 V). */

    /* Derivative variables. */
    uint32_t sampleId;          /* Current sample ID. */
    uint32_t numSamples;        /* Number of samples in the experiment. */
    uint32_t testDuration;
};

/** Function definitions. */
/** Indicator Management. */
static void cycleLed(DigitalOut *dout, uint8_t numCycles, std::chrono::milliseconds delayMs);
static void heartbeat(void);

/** Sampling sensor data. */
static void performTest(void);

/** Communication Input Processing. */
static void pollSerial(void);
static void pollCan(void);
static uint16_t setProfile(char *buf, struct profile *profile);

/** Processing. */
static void processVoltageResult(uint32_t sampleId, float data);
static void processCurrentResult(uint32_t sampleId, float data);
static void processResult(uint16_t msgId, enum measurementType mType, uint32_t sampleId, uint32_t value);
static void processError(uint16_t msgId, uint16_t errorCode, uint16_t errorContext);

/** Error Handling. */
static void setError(uint16_t msgId, uint16_t errCode, uint16_t errorContext);
static void errorLoop(void);