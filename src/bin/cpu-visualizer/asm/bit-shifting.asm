; lsr, asl, rol, ror commands work in the A register. Test those here.

main:
    ; Results stored in zero page: 0x00
    jsr prep_test;
    jsr lsr_register_a;
    jsr finish_test;

    ; Results stored in zero page: 0x01
    jsr prep_test;
    jsr asl_register_a;
    jsr finish_test;

    ; Results stored in zero page: 0x02
    jsr prep_test;
    jsr rol_register_a;
    jsr finish_test;

    ; Results stored in zero page: 0x03
    jsr prep_test;
    jsr ror_register_a;
    jsr finish_test;

    ; Results stored in zero page: 0x04
    jsr prep_test;
    jsr lsr_register_a_2;
    jsr finish_test;

    kil

prep_test:
    ; Set the carry bit to 1, and register A to 0.
    lda #$ff
    adc #$01
    rts

finish_test:
    ; Store the results incrementally in the zero page
    sta $00,x
    ; Move the carry bit to A
    lda #$00
    adc #$00
    ; Store this on the zero page as well
    sta $20,x
    inx
    rts

; Logical shift right
;; >> without carry being applied
lsr_register_a:
    lda #%10101010
    lsr A
    ; Expect A: %01010101, C: 0
    ;            ^ no carry applied
    rts

; Logical shift right
;; >> without carry being applied
lsr_register_a_2:
    lda #%01010101
    lsr A
    ; Expect A: %00101010, C: 1
    ;            ^            ^ bit 1 was shifted into carry
    ;            no carry applied
    rts

; Arithmetic Shift left
; << without carry
asl_register_a:
    lda #%10101010
    asl A
    ; Expect A: %01010100, C: 1
    ;                   ^     ^ bit 7 was shifted into carry
    ;                   The carry bit was not applied to the beginning
    rts

; Rotate left
;; << with carry
rol_register_a:
    lda #%10101010
    rol A
    ; Expect A: %01010101, C: 1
    ;                   ^ Carry bit was applied
    rts

; Rotate right
;; >> with carry
ror_register_a:
    lda #%10101010
    ror A
    ; Expect A: %11010101, C: 0
    ;            ^ Carry bit was applied
    rts
