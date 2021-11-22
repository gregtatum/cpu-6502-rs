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

    kil

prep_test:
    ; Set the carry bit to 1, and register A to 0.
    lda #$ff
    adc #$01
    rts

finish_test:
    ; Store the results incrementally in the zero page
    sta $00,x
    inx
    rts

;; >> without carry
lsr_register_a:
    lda #%10101010
    lsr A
    ; Expect %gs
    rts

; << without carry
asl_register_a:
    lda #%10101010
    asl A
    ; Expect %01010100
    rts

; Rotate left
;; << with carry
rol_register_a:
    lda #%10101010
    rol A
    ; Expect %01010101
    rts

; Rotate right
;; >> with carry
ror_register_a:
    lda #%10101010
    ror A
    ; Expect %11010101
    rts
