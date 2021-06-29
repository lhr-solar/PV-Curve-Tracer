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


#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>
#include <stdio.h>

class Fifo {
    private:
        char * buffer;
        size_t head;
        size_t tail;
        size_t maxCapacity;
        size_t usedCapacity;

    public:
        Fifo(char * buffer, size_t maxCapacity) {
            this->buffer = buffer;
            this->head = 0;
            this->tail = 0;
            this->maxCapacity = maxCapacity;
            this->usedCapacity = 0;
        }

        void clear(void) {
            head = 0;
            tail = 0;
            usedCapacity = 0;
        }

        bool isFull(void) { return (((head + 1) % maxCapacity) == tail) && (usedCapacity == maxCapacity); }

        bool isEmpty(void) { return (head == tail) && (usedCapacity == 0);}

        size_t getUsedCapacity(void) { return usedCapacity; }

        /**
         * Peek at the first len characters in the buffer.
         * 
         * @param[out] buffer External buffer to read characters into.
         * @param[in] len Length of external buffer and number of characters to read.
         * @param[in] idx Starting index to read at.
         * @return Number of characters looked at in the buffer.
         */
        size_t peek(char* buffer, size_t len) {
            if (isEmpty()) return 0;

            size_t charsRead = 0;
            uint16_t i = tail;
            while (i != head && charsRead < (len-1)) {
                buffer[charsRead] = this->buffer[i];
                i = (i + 1) % maxCapacity;
                ++charsRead;
            }

            return charsRead;
        }

        /**
         * Enqueues a character in the buffer if the buffer is not full.
         * 
         * @param[in] inp Input character to put into the buffer.
         * @return true if buffer is not full.
         */
        bool enqueue(char inp) {
            if (isFull()) return false;
                        
            buffer[head] = inp;
            head = (head + 1) % maxCapacity;
            ++usedCapacity;

            return true;
        }

        /**
         * Dequeue and returns first character in the buffer if the buffer is not empty.
         * 
         * @param[out] inp Output character address to read character into.
         * @return true if buffer is not empty.
         */
        bool dequeue(char& inp) {
            if (isEmpty()) return false;
                        
            inp = buffer[tail];
            buffer[tail] = '\0';
            tail = (tail + 1) % maxCapacity;
            --usedCapacity;

            return true;
        }
        
        void _printQueue(void) {
            printf("Head:%i\tTail:%i\t", head, tail);
            for (size_t i = 0; i < maxCapacity; ++i) {
                printf("[%c%i:%c%c]", i == tail ? '*' : '_', i, buffer[i], i == head ? '*' : '_');
            }
            printf("\n");
        }
};


// int testMain() {
//     char buffer[5] = { 0 };
//     Fifo fifo = Fifo(buffer, 5);
//     fifo._printQueue();
//     fifo.enqueue('h');
//     fifo.enqueue('e');
//     fifo.enqueue('y');
//     fifo.enqueue('o');
//     if (!fifo.enqueue('!')) printf("Queue is full!\n");
    
//     char out;
//     fifo.dequeue(out);
//     fifo.dequeue(out);
//     fifo.dequeue(out);
//     fifo.dequeue(out);
//     if (!fifo.dequeue(out)) printf("Queue is empty!\n");
    
//     fifo.enqueue('a');
//     fifo.enqueue('b');
//     fifo.enqueue('c');

//     char buffer2[5] = { 0 };
//     printf("Read %i chars: %s\n", fifo.peek(buffer2, 5), buffer2);
//     return 0;
// }
