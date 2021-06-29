/**
 * Maximum Power Point Tracker Project
 *
 * File: ComIds.cpp
 * Author: Matthew Yu
 * Organization: UT Solar Vehicles Team
 * Created on: May 25th, 2021
 * Last Modified: 06/29/21
 *
 * File Description: This file defines valid MSG IDs for the Array/MPPT system.
 */
#pragma once

#define SUNSCTR_1_ARR_V_SP      0x600
#define SUNSCTR_1_ARR_V_MEAS    0x601
#define SUNSCTR_1_ARR_C_MEAS    0x602
#define SUNSCTR_1_BATT_V_MEAS   0x603
#define SUNSCTR_1_BATT_C_MEAS   0x604
#define SUNSCTR_1_EN_DIS        0x605
#define SUNSCTR_1_FAULT         0x606

#define SUNSCTR_2_ARR_V_SP      0x610
#define SUNSCTR_2_ARR_V_MEAS    0x611
#define SUNSCTR_2_ARR_C_MEAS    0x612
#define SUNSCTR_2_BATT_V_MEAS   0x613
#define SUNSCTR_2_BATT_C_MEAS   0x614
#define SUNSCTR_2_EN_DIS        0x615
#define SUNSCTR_2_FAULT         0x616

#define BLKBDY_TEMP_MEAS        0x620
#define BLKBDY_IRRAD_1_MEAS     0x630
#define BLKBDY_IRRAD_2_MEAS     0x631
#define BLKBDY_EN_DIS           0x632
#define BLKBDY_FAULT            0x633

#define CRVTRCR_INP_PROFILE     0x640
#define CRVTRCR_RESULT          0x641
#define CRVTRCR_FAULT           0x642
#define CRVTRCR_VOLT_MEAS       0x643
#define CRVTRCR_CURR_MEAS       0x644

#define INVALID_MSG_ID          0xEEEE
