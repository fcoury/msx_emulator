; TestVDP.asm

; Initialize VDP constants
VDP_WRITE_REG    EQU 0x99
VDP_WRITE_VRAM   EQU 0x98
VDP_READ_VRAM    EQU 0x98

; Initialize data
    LD A, 0x0F
    LD B, 0x22
    LD C, 0x55
    LD D, 0x88

; Set screen mode to Screen 2
    LD A, 0x80 | 0x06
    OUT (VDP_WRITE_REG), A
    LD A, 0x81 | 0x00
    OUT (VDP_WRITE_REG), A

; Initialize VRAM address to 0x0000
    LD HL, 0x0000
    LD A, H
    OUT (VDP_WRITE_VRAM), A
    LD A, L
    OUT (VDP_WRITE_VRAM), A

; Fill VRAM with a pattern
    LD B, 0xFF
    LD C, 0x98
    LD DE, 0x0800
FillVRAM:
    OUTI
    DEC DE
    LD A, D
    OR E
    ; JP NZ, FillVRAM
    HALT

; Read data back from VRAM
    LD HL, 0x0000
    LD A, H
    OUT (VDP_READ_VRAM), A
    LD A, L
    OUT (VDP_READ_VRAM), A
    IN A, (VDP_READ_VRAM)
    LD (0x8000), A

    HALT
