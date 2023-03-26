org 0x0000

; Initialize registers
LD A, 0x01
LD B, 0x02
LD C, 0x03

; Simple arithmetic
ADD A, B
ADC A, C

; HALT
HALT
