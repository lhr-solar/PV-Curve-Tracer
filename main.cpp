/**
 * Project: PV Curve Tracer Board
 * File: main.cpp
 * Author: Matthew Yu (2021).
 * Organization: UT Solar Vehicles Team
 * Created on: 05/29/21
 * Last Modified: 06/29/21
 * File Description: This file describes the operation and execution of the
 * PV Curve Tracer board for the UT LHR Solar Vehicles Team. 
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
#include "main.hpp"

/** LEDs. */
DigitalOut ledHeartbeat(D3);
DigitalOut ledScanning(LED_SCANNING);
DigitalOut ledCanTx(CAN_TX);
DigitalOut ledCanRx(CAN_RX);
DigitalOut ledError(LED_ERROR);

/** Tickers. */
LowPowerTicker tickHeartbeat;

/** Sensors. */
AnalogIn sensorVoltage(ADC_VOLTAGE);
AnalogIn sensorCurrent(ADC_CURRENT);

/** DAC. */
AnalogOut controlDac(DAC_CONTROL);

/** Comm. */
static BufferedSerial serialPort(USB_TX, USB_RX);
static CAN canPort(CAN_RX, CAN_TX);

/** Globals */
struct profile testProfile = {
    .complete = false,
    .testRegime = profile::NO_REGIME,
    .sampleId = 0
};
struct result results[4] = {
    { .sensorType = result::VOLTAGE     },
    { .sensorType = result::CURRENT     },
    { .sensorType = result::IRRADIANCE  },
    { .sensorType = result::TEMPERATURE }
};
uint16_t errorCode = ERR_NONE;

/** Processing structures. */
static EventQueue queue(QUEUE_SIZE * EVENTS_EVENT_SIZE);

/** Main routine. */
int main() {
    /* Setup serial comm. */
    serialPort.set_baud(9600);
    serialPort.set_format(
        8,                      /* bits */ 
        BufferedSerial::None,   /* parity */ 
        1                       /* stop bit */ 
    );

    /* Cycle LEDs. */
    cycleLed(&ledHeartbeat, 4, std::chrono::milliseconds(100));
    cycleLed(&ledScanning, 4, std::chrono::milliseconds(100));
    cycleLed(&ledCanTx, 4, std::chrono::milliseconds(100));
    cycleLed(&ledCanRx, 4, std::chrono::milliseconds(100));
    cycleLed(&ledError, 4, std::chrono::milliseconds(100));

    /* Set a heartbeat toggle for 0.5 Hz. */
    tickHeartbeat.attach(&heartbeat, chrono::milliseconds(1000));

    /* Start threads for processing data and capturing. */
    Thread threadProcessing, threadTesting;
    threadProcessing.start(callback(&queue, &EventQueue::dispatch_forever));
    threadTesting.start(performTest);

    /* Main thread looks for messages. */
    while (true) {
        pollSerial();
        pollCan();
        ThisThread::sleep_for(chrono::milliseconds(100));
    }
}

/** Indicator Management. */
static void heartbeat(void) { ledHeartbeat = !ledHeartbeat; }
static void cycleLed(DigitalOut *dout, uint8_t numCycles, std::chrono::milliseconds delayMs) {
    for (uint8_t i = 0; i < numCycles; ++i) {
        *dout = 1;
        ThisThread::sleep_for(delayMs);
        *dout = 0;
        ThisThread::sleep_for(delayMs);
    }
}

/** Sampling sensor data. */
static void sampleOnboardSensors(struct profile *profile, struct result *resultVoltage, struct result *resultCurrent) {
    resultVoltage->value = sensorVoltage;
    resultVoltage->sampleId = profile->sampleId;
    resultVoltage->isProcessed = false;

    resultCurrent->value = sensorCurrent;
    resultCurrent->sampleId = profile->sampleId;
    resultCurrent->isProcessed = false;
}
static void performTest(void) {
    while (true) {
        /* Wait for profile to begin. */
        if (testProfile.complete) {
            /* Cycle scanning LED for 3 seconds. */
            cycleLed(&ledScanning, 3, chrono::milliseconds(250));

            testProfile.sampleId = 0;
            testProfile.numSamples = (testProfile.voltageEnd - testProfile.voltageStart)/testProfile.voltageResolution;
    
            /* Turn on scanning LED and perform test. */
            ledScanning = 1;
            do {
                /* Set DAC output. Multiplied by 5x in HW. */
                controlDac = (float) testProfile.sampleId * testProfile.voltageResolution + testProfile.voltageStart;

                /* When DAC output is as expected, wait X ms for system to stabilize. */
                ThisThread::sleep_for(std::chrono::milliseconds(15)); /* TODO: make dependent on voltage step and mode. */

                /* Capture sensor data and post to queue. */
                sampleOnboardSensors(&testProfile, &results[VOLTAGE_IDX], &results[CURRENT_IDX]);
                queue.call(processResult, CRVTRCR_VOLT_MEAS, &results[VOLTAGE_IDX]);
                queue.call(processResult, CRVTRCR_CURR_MEAS, &results[CURRENT_IDX]);

                /* Idle until queue has processed the events. */
                while (!results[VOLTAGE_IDX].isProcessed || !results[CURRENT_IDX].isProcessed)
                    ThisThread::sleep_for(std::chrono::milliseconds(5));

                testProfile.sampleId++;
            } while (testProfile.sampleId < testProfile.numSamples);  

            /* Turn off scanning LED and set testProfile back to false. */      
            ledScanning = 0;
            testProfile.complete = false;
        }
        ThisThread::sleep_for(std::chrono::milliseconds(2500));
    }
}

/** Communication Input Processing. */
static void pollSerial(void) {
    #define MAX_BUFFER_SIZE 3 * 8
    #define DATA_TRANSFER_SIZE 4
    
    static char data[DATA_TRANSFER_SIZE] = { 0 };       /* Buffer for reading and peeking at data. */
    static char buffer[MAX_BUFFER_SIZE];                /* Buffer for fifo storage. */
    static Fifo fifo = Fifo(buffer, MAX_BUFFER_SIZE);   /* Fifo for message capture. 3x the largest message size, 8 bytes. */
        
    /* If there is a an opportunity to read a byte, attempt to take it. */
    if (!fifo.isFull()) {
        if (serialPort.read(data, 1)) {
            printf("Read: %02x\n", data[0]);
            fifo.enqueue(data[0]);
        }
    }
    
    /* Peek into the FIFO for the first 3 bytes. */
    if (fifo.peek(data, DATA_TRANSFER_SIZE) == 3) {
        if (data[0] != PRELUDE) {
            char throwawayByte;
            if (!fifo.dequeue(throwawayByte)) {
                /* Fault if fifo cannot discard the first byte. */
                setError(CRVTRCR_FAULT, ERR_INVALID_FIFO_DEQUEUE, 0x00);
            }
            return;
        }
        
        /* Handle data based on message ID. */
        uint16_t msgId = ((data[1] << 8) | (data[2] & 0xF0)) >> 4;
        switch (msgId) {
            case CRVTRCR_INP_PROFILE: 
                #define CRVTRCR_INP_PROFILE_NUM_BYTES 8
                if (fifo.getUsedCapacity() >= CRVTRCR_INP_PROFILE_NUM_BYTES) {
                    /* Read CRVTRCR_INP_PROFILE_NUM_BYTES chars and begin
                       parsing. */
                    char input[CRVTRCR_INP_PROFILE_NUM_BYTES] = { 0 };
                    for (uint8_t i = 0; i < CRVTRCR_INP_PROFILE_NUM_BYTES; ++i) {
                        fifo.dequeue(input[i]);
                    }

                    /* Validate the profile. */
                    uint16_t errCode = setProfile(input, &testProfile);
                    if (errCode) setError(CRVTRCR_FAULT, errCode, 0x00);
                    else testProfile.complete = true;
                }
                #undef CRVTRCR_INP_PROFILE_NUM_BYTES
                break;
            
            /* These should never be received. Throw an error. */
            case CRVTRCR_RESULT:
            case CRVTRCR_FAULT: 
            default:
                setError(CRVTRCR_FAULT, ERR_UNEXPECTED_MSG_ID, 0x00);
                break;
        }
    }

    #undef DATA_TRANSFER_SIZE
    #undef MAX_BUFFER_SIZE
}
static void pollCan(void) {
    static CANMessage msg;

    if (canPort.read(msg)) {
        /* Handle data based on message ID. */
        uint16_t msgId = msg.id;
        switch (msgId) {
            case BLKBDY_TEMP_MEAS:
                if (testProfile.complete) {
                    /* TODO: for now, we'll wipe the RTD ID from the temperature 
                       sensor measurement. Let's add support for this later. */
                    msg.data[4] = '\0';
                    /* TODO: test reinterpret cast is correct based on endianness. */
                    results[TEMPERATURE_IDX].value = *(reinterpret_cast<float*>(msg.data)) / 1000.0;
                    results[TEMPERATURE_IDX].sampleId = testProfile.sampleId;
                    queue.call(processResult, msgId, &results[TEMPERATURE_IDX]);
                }
                break;
            case BLKBDY_IRRAD_1_MEAS:
                if (testProfile.complete) {
                    results[IRRADIANCE_IDX].value = *(reinterpret_cast<float*>(msg.data)) / 1000.0;
                    results[IRRADIANCE_IDX].sampleId = testProfile.sampleId;
                    queue.call(processResult, msgId, &results[IRRADIANCE_IDX]);
                }
                break;
            case BLKBDY_IRRAD_2_MEAS:
                if (testProfile.complete) {
                    results[IRRADIANCE_IDX].value = *(reinterpret_cast<float*>(msg.data)) / 1000.0;
                    results[IRRADIANCE_IDX].sampleId = testProfile.sampleId;
                    queue.call(processResult, msgId, &results[IRRADIANCE_IDX]);
                }
                break;
            case BLKBDY_FAULT:
                setError(msgId, msg.data[0], msg.data[1]);
                break;

            /* These should never be received. Throw an error. */
            case BLKBDY_EN_DIS:
            default:
                setError(CRVTRCR_FAULT, ERR_UNEXPECTED_MSG_ID, 0x00);
                break;
        }
    }
}
static uint16_t setProfile(char *buf, struct profile *profile) {
    #define UPPER_NIBBLE 0xF0
    #define LOWER_NIBBLE 0x0F


    /* Byte 3, most significant nibble (MSN) is Test Regime Type. */
    profile->testRegime = (enum profile::regime)((buf[3] & UPPER_NIBBLE) >> 4);
    if (profile->testRegime == profile::NO_REGIME || profile->testRegime >= profile::RESERVED1) {
        return ERR_INVALID_PROFILE;
    }

    /* Byte 3 LSN, 4 is Start Voltage * 1000. */
    profile->voltageStart = (float) (((buf[3] & LOWER_NIBBLE) << 8) | buf[4]) / 1000.0;
    if (profile->voltageStart < 0.0 || profile->voltageStart > 3.3) {
        return ERR_INVALID_VOLTAGE_START;
    }

    /* Byte 5, 6 MSN is End Voltage * 1000. */
    profile->voltageEnd = (float) ((buf[5] << 4) | ((buf[6] & UPPER_NIBBLE) >> 4)) / 1000.0;
    if (profile->voltageEnd < 0.0 || profile->voltageEnd > 3.3) {
        return ERR_INVALID_VOLTAGE_END;
    }

    if (profile->voltageStart > profile->voltageEnd) {
        return ERR_INVALID_VOLTAGE_CONSISTENCY;
    }

    /* Byte 6 LSN, 7 is Voltage Resolution * 1000. */
    profile->voltageResolution = (float) (((buf[6] & LOWER_NIBBLE) << 8) | buf[7]) / 1000.0;
    if (profile->voltageResolution <= 0.0 || profile->voltageResolution > 1.0) {
        return ERR_INVALID_VOLTAGE_RESOLUTION;
    }

    return ERR_NONE;

    #undef UPPER_NIBBLE
    #undef LOWER_NIBBLE
}
static uint16_t setTempMeas(char *buf, struct profile *profile) {
    return ERR_NONE;
}
static uint16_t setIrradMeas(char *buf, struct profile *profile) {
    return ERR_NONE;
}

/** Outbound Message Processing. */
void processResult(uint16_t msgId, struct result *result) {
    printf(
        "%02x%03x%01x%03x%05x",
        PRELUDE,
        msgId,
        result->sensorType,
        result->sampleId,
        (uint32_t)(result->value * 1000)
    );
    result->isProcessed = true;
}
void processError(uint16_t msgId, uint16_t errorCode, uint16_t errorContext) {
    printf(
        "%02x%03x%03x%04x", 
        PRELUDE,
        msgId,
        errorCode,
        errorContext
    );
}

/** Error Handling. */
static void setError(uint16_t msgId, uint16_t errCode, uint16_t errorContext) {
    /* Set the error code to force other threads to halt. */
    errorCode = errCode;
    testProfile.complete = false;
    
    /* Tell the processing thread to submit an exception message. */
    queue.call(processError, msgId, errCode, errorContext);
    
    /* Error loop this thread. */
    errorLoop();
}
static void errorLoop(void) {
    /* Turn on error LED. */
    ledError = 1;

    /* Loop forever. */
    while (true);
}
