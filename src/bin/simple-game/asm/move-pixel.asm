; $00 - x position
; $01 - y position
; $02-03 - Pointer to memory location

example_call_to_write_pixel:
    ldx #$01 ; color
    ldy #31  ; x position
    lda #31  ; y position
    jsr write_pixel
    sleep:
    jmp sleep


; Set A to the pixel value.
; ldx: color
; ldy: x position
; lda: y position
write_pixel:
    ; Put the Y position in the most significant byte.
    pha     ; Remember the Y.
    lsr     ; Shift 0001_1111 to 0000_0011
    lsr     ;
    lsr     ;
    sta $11 ; Store it.

    ; Build the least significant byte from Y.
    pla     ; Get the Y position again
    asl     ; Shift 0001_1111 to 1110_0000
    asl     ;
    asl     ;
    asl     ;
    asl     ;
    sta $10 ; Store it.

    ; Get the X portion.
    tya     ; Get the X position now
    ora $10 ; Combine it with the Y bits
    sta $10 ; Store the final result.

    ; Write the color out.
    txa         ; Get the color
    ldx #$00    ; Don't offset
    sta ($10,x) ; Save to our address

    ; We're done!
    rts
