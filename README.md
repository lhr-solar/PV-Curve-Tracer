# PV Curve Tracer

This repository contains the source code for driving the PCB defined in
[Array-CurveTracerPCB
repository](https://github.com/lhr-solar/Array-CurveTracerPCB). It is meant to
be used with the Blackbody boards: [Array-RTDPCB
repository](https://github.com/lhr-solar/Array-RTDPCB) and
[Array-StandaloneIrradiancePCB
repository](https://github.com/lhr-solar/Array-StandaloneIrradiancePCB), and
data is siphoned by the Heliocentric visualizer application -currently a fork of
[DeSeCa](https://github.com/dimembermatt/DeviceSerialCapture), to be merged with
[ArraySimulation](https://github.com/lhr-solar/Array-Simulation). The data
communication protocol with Heliocentric is described below. 

---
## Requirements
To compile and run the code, you need a linked version of mbed OS 6.

Additional requirements include pulling at least release 0.1.0 of
[Mbed-Shared-Components](https://github.com/lhr-solar/Mbed-Shared-Components);
this release contains the communication message IDs and error IDs that are used
in the firmware.

---
## TODO
- Move Errors, ComIds to Mbed-Shared-Components.
- Add exception handling to bad sensor data (post-calibration).
- Migrate serial input processing, CAN input processing to class.
- Create common com input processing class that encompasses the above.
- Look at common eventQueue com output processing class that encompasses the above.

---
## Operation

The PV Curve Tracer operates on a three thread scheme: 
- A primary thread doing initialization and *input processing*. This includes
  capturing messages from other devices (such as the Blackbody line of sensor
  boards and the host PC) and managing them.
- A secondary thread doing experiments. This thread is always active, but will
  not execute an experiment unless a **test profile** is defined. Once a test
  profile is defined, the thread will begin interrogating the hardware and will
  then post its results to be transmitted.
- A tertiary thread that acts as an event queue. Events are posted into this
  queue by the primary and secondary threads, and the event queue will process
  them in order of being received. Typically these events have soft real-time
  deadlines, and/or cannot execute in an ISR context. A good example of such
  events is outbound communication.

### Nominal operation loop

In a typical experiment, the secondary thread and tertiary thread starts off
idle. The primary thread begins receiving serial data across USART/USB from the
user PC, and upon receiving a valid command (defined in section **Serial
communication protocol with PC**), it notifies the secondary thread through a
semaphore.

When the secondary thread is notified that is has a valid profile to begin
experimenting with, it signals to the user that it will begin the experiment by
cycling the SCANNING LED thrice. It then turns the SCANNING LED on and begins
the following cycle until an internal endpoint is reached:

1. The DAC is updated with an output voltage.
2. The system waits a period of time for the input sensor data to stabilize.
3. The onboard current and voltage sensors sample the input data.
4. The data is sampled, calibrated, and packaged into a result.
5. The tertiary thread queue is notified to process the result and send it to
   the user.

When reaching the endpoint, the SCANNING LED turns off and the secondary thread
goes back into an idle loop until the next valid profile.

During the loop execution, messages from the Blackbody sensor boards may arrive
and be processed by the primary thread. These messages can be several things,
but typically they are measurements from either irradiance or temperature
sensors. Upon receiving these measurements, the primary thread assigns a logical
time stamp to them (associated with the most recent secondary thread
interrogation) and also posts them to the tertiary thread queue to manage.

The tertiary thread will simply loop the event queue execution until the end of
the device lifecycle.

This three thread scheme was selected in order to maintain 2 primary objectives:
1. Reasonable firm real-time experimentation for thread 2. Allocating serial
   processing work (which is blocking and time-consuming) to thread 3 allows the
   thread to continue relatively unimpeded.
2. Allow for the reasonable serving of inbound communications. Similarly to
   thread 2, thread 1 must be available to serve inbound messages from various
   messaging schemes within a reasonable amount of time. Otherwise messages can
   be lost and/or corrupted which will lead to software exceptions and overall
   functionality degradation.

### Error handling

The firmware also has a message encoding and error handling scheme. Software
exceptions and critical errors are either handled or cause a software halt,
spinning in an error loop in one or several threads and turning on the ERROR
LED. Prior to the error loop, an error message is sent outbound to the relevant
parties containing a predefined error code indicating what caused the issue.

---
## Communication

Communication between the PV Curve Tracer and other parties are defined in the
following sections. Typically, messages with IDs that are not explicitly defined
for each context will be treated as software exceptions and reported to the user.

## CAN communication protocol.

The CAN communication protocol concerns the transmission of messages from the
Blackbody boards. All messages are input sans the Blackbody Enable/Disable
command; the user, through the PV Curve Tracer can disable external sensor input
messages through this command.

| Name                                      | Direction | ID    | Data Width [L:S]   | Data Type                   | Frequency |
|-------------------------------------------|-----------|-------|--------------------|-----------------------------|-----------|
| Blackbody Temperature Sensors Measurement | I         | 0x620 | [39 : 32] [31 : 0] | RTD ID; C, signed float*100 | 2 Hz      |
| Blackbody Irradiance Sensor 1 Measurement | I         | 0x630 | [31 : 0]           | W/m^2, signed float*100     | 10 Hz     |
| Blackbody Irradiance Sensor 2 Measurement | I         | 0x631 | [31 : 0]           | W/m^2, signed float*100     | 10 Hz     |
| Blackbody Enable/Disable                  | O         | 0x632 | [0 : 0]            | 1: Halt, 0: Restart         | Async     |
| Blackbody Board Fault                     | I         | 0x633 | [7 : 0]            | Error ID, enum              | Async     |

### Serial alternative communication protocol.
In the event of a CAN failure or testing, the following serial communication
protocol, separate to the serial communication protocol with PC, can be used.

Using the same table as above:
- 1 byte 0xFF prelude.
- 2 bytes of ID.
- x bytes according to the data width.

---
## Serial communication protocol with PC

This protocol is purely for communication with a user over USB USART. Three
message types are currently supported, and they follow the following format:

- 1 byte 0xFF prelude.
- 1.5 bytes of the CAN ID or MSG ID.
- x.y bytes of resulting data fields.

### PV Curve Tracer input profile.
PC to Curve Tracer.
```js
Bitmap                      | Contents                          | Data Width
[63:56] - byte 7            | 0xFF                              | 0xFF
[55:48] - byte 6            | CAN ID/MSG ID                     | 0xFFF
[47:44] - byte 5, nibble 2  | CAN ID/MSG ID                     | 
[43:40] - byte 5, nibble 1  | RESERVED                          | 0xF
[39:36] - byte 4, nibble 2  | Test Regime Type                  | 0xF
[35:32] - byte 4, nibble 1  | Start Voltage (x.yyy * 1000)      | 0xFFF
[31:24] - byte 3            | Start Voltage                     |
[23:16] - byte 2            | End Voltage (x.yyy * 1000)        | 0xFFF
[15:12] - byte 1, nibble 2  | End Voltage                       |
[11:8]  - byte 1, nibble 1  | Voltage Resolution (x.yyy * 1000) | 0xFFF
[7:0]   - byte 0            | Voltage Resolution                |
```

### PV Curve Tracer result.
Curve Tracer to PC.
```js
Bitmap                      | Contents                          | Data Width
[55:48] - byte 6            | 0xFF                              | 0xFF
[47:40] - byte 5            | CAN ID/MSG ID                     | 0xFFF
[39:36] - byte 4, nibble 2  | CAN ID/MSG ID                     |
[35:32] - byte 4, nibble 1  | Measurement Type                  | 0xF
[31:24] - byte 3            | Sample ID                         | 0xFFF
[23:20] - byte 2, nibble 2  | Sample ID                         |
[19:16] - byte 2, nibble 1  | Value (xxxx.yyy * 1000)           | 0xFFFFF
[15:8]  - byte 1            | Value                             |
[7:0]   - byte 0            | Value                             |
```

### PV Curve Tracer exception.
Curve Tracer to PC.
```js
Bitmap                      | Contents                          | Data Width
[47:40] - byte 5            | 0xFF                              | 0xFF
[39:32] - byte 4            | CAN ID/MSG ID                     | 0xFFF
[31:28] - byte 3, nibble 2  | CAN ID/MSG ID                     | 
[27:24] - byte 3, nibble 1  | Error Code                        | 0xFFF
[23:16] - byte 2            | Error Code                        |
[15:8]  - byte 1            | Error Context                     | 0xFFFF
[7:0]   - byte 0            | Error Context                     |
```

## Example user input stream

```rust
0xFF, 0x64, 0x2F, 0x10, 0x00, 0x2B, 0xC0, 0x32

/* Concatenates to: */
0xFF642F10002BC032

/* Which can be split by sections into: */
0xFF_642_F_1_000_2BC_032

/**
 * Which represents:
 * 0xFF  - Prelude
 * 0x642 - MSG ID
 * 0xF   - RESERVED
 * 0x1   - Voltage test
 * 0x000 - 0.000 Start voltage
 * 0x2BC - 0.700 End voltage
 * 0x032 - 0.050 Voltage resolution
 */
```

The recommended tool to send this bytestream is DeSeCa, since some characters, 
like 0xFF, cannot be represented in ASCII format and sent via serial console.