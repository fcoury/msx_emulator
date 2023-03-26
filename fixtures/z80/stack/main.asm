org 0x0000

; Initialize registers and memory
LD A, 5       ; 3E 05
LD B, 7       ; 06 07
LD DE, 0x8000 ; 11 00 80

; Call subroutine
CALL 0x000B   ; CD 0B 00

; Halt the program
HALT          ; 76

; Subroutine (0x0010):
; Add A and B, store the result at (DE)
; Preserve registers using PUSH and POP

; Save AF and BC on the stack
PUSH AF       ; F5
PUSH BC       ; C5

; Add A and B, store result in A
ADD A, B      ; 80

; Save result at (DE)
LD (DE), A    ; 12

; Restore BC and AF from the stack
POP BC        ; C1
POP AF        ; F1

; Return from subroutine
RET           ; C9
