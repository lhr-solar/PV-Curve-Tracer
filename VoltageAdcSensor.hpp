/**
 * Project: PV Curve Tracer Board
 * File: VoltageAdcSensor.hpp
 * Author: Matthew Yu (2021).
 * Organization: UT Solar Vehicles Team
 * Created on: 07/04/21
 * Last Modified: 07/04/21
 * File Description: Header file that implements a voltage sensor.
 */

/** Includes. */
#include <Sensor/Sensor.hpp>


class VoltageAdcSensor : public Sensor {
    private:
        AnalogIn * sensorVoltage;

    public:
        VoltageAdcSensor(AnalogIn *sensor) : Sensor() {
            sensorVoltage = sensor;
        }
    
    private:
        virtual void _sampleData(void) {
            float tempSensorVoltage = *sensorVoltage;

            /* TODO: calibration here. */
            
            data = tempSensorVoltage;
        }
};
