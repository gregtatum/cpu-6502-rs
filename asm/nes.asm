; The following bytes are the magic initialization values.
; https://en.wikibooks.org/wiki/NES_Programming/Initializing_the_NES
.byte "NES"
.byte $1a

; Number of PRG-ROM blocks
.byte $01

; Number of CHR-ROM blocks
.byte $01

; ROM control bytes: Horizontal mirroring, no SRAM or trainer, Mapper #0
.byte $00, $00

; Filler
.byte $00,$00,$00,$00,$00,$00,$00,$00
