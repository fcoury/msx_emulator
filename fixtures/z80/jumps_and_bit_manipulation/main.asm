; Test.asm

; Initialize data
    LD A, 0x0F
    LD B, 0x22
    LD C, 0x55
    LD D, 0x88

; Test BIT instruction
    BIT 1, A    ; Test bit 1 of A (should set Z flag)
    BIT 6, B    ; Test bit 6 of B (should not set Z flag)

; Test conditional jump
    JP NZ, SetBitInC

; If Z flag is not set, we should not reach this point
    XOR A       ; Reset A to 0
    HALT

SetBitInC:
    SET 1, C    ; Set bit 1 in C

; Test RES instruction
    RES 3, D    ; Reset bit 3 in D

; Test PUSH and POP instructions
    PUSH BC
    POP DE

; Test CALL and RET instructions
    CALL IncrementA
    JP End

IncrementA:
    INC A
    RET

End:
    HALT
