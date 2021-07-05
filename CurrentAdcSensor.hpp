/**
 * Project: PV Curve Tracer Board
 * File: CurrentAdcSensor.hpp
 * Author: Matthew Yu (2021).
 * Organization: UT Solar Vehicles Team
 * Created on: 07/04/21
 * Last Modified: 07/04/21
 * File Description: Header file that implements a current sensor.
 */

/** Includes. */
#include <Sensor/Sensor.hpp>

class CurrentAdcSensor : public Sensor {
    private:
        AnalogIn * sensorCurrent;

    public:
        CurrentAdcSensor(AnalogIn *sensor) : Sensor() {
            sensorCurrent = sensor;
        }
    
    private:
        virtual void _sampleData(void) {
            float tempSensorCurrent = *sensorCurrent;

            /* TODO: calibration here. */
            
            data = tempSensorCurrent;
        }
};
