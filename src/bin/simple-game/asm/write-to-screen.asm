write_to_screen:
    ; Cycle through the 10 colors.
    cmp #$10
    bne endif
        lda #$00
    endif:

    ; Store pixels on the screen
    sta $0500,x

    inx       ; Move on to the next pixel
    adc #$01  ; Cycle to the next color.
    jmp write_to_screen
