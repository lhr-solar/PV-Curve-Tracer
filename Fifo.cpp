/**
 * Project: mbed-shared-components
 * File: fifo.cpp
 * Author: Matthew Yu (2021).
 * Organization: UT Solar Vehicles Team
 * Created on: 06/26/21
 * Last Modified: 06/29/21
 * File Description: This file declares and implements a Fifo class, which
 * can be used for message passing.
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
