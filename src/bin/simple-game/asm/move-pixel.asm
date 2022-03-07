; TODO - this is not working, as it was based on a different version of simple game.

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
    ; Build the address inside of page $02 from Y.
    asl     ; Shift 0001_1111 to 1111_1000
    asl     ;
    asl     ;
    sta $10 ; Store it.

    ; Get the X portion.
    tya     ; Get the X position now
    pha     ; Remember X
    lsr     ; Shift 0001_1111 to 0000_0111
    lsr     ;

    ora $10 ; Combine it with the Y bits
    sta $10 ; Store the final result.

    ; The video memory is stored on page $02.
    lda #$02
    sta $11

    txa ; Get the color
    tay ; We need the X register, transfer the color to the Y register.
    pla ; Get the X position again
    and #$03; ; Isolate the bit offset amount. 0b0000_0011

    ; Write the color out.
    tax         ; Store the bit offset into the X
    tya         ; Get the color from the Y register
    pha
    lda #$fc    ; Load a bit mask into A, this will get offset as well.
                ; 0b1111_1100
    tay         ; Move it into Y.
    pla         ; Restore the color.


    ; A - Color to offset
    ; X - Amount to offset
    ; Y - Mask offset
    write_pixel_loop:
        ; Loop 0-4 times.
        cpx #$00
        beq write_pixel_done
        dex ; Decrement X for the loop.

        ; Offset the color.
        asl ; Shift the color into the proper bit position.
        asl
        pha ; Push the color onto the stack.

        ; Offset the mask.
        sec ; Set the carry bit so 1 will be rolled on.
        tya ; Load the mask.
        rol ; Offset it too, but apply the carry bit.
        rol
        tay
        pla ; Restore A.
        jmp write_pixel_loop
    write_pixel_done:

    ; The color is correct, we need to apply it to the target.
    pha         ; Push the shifted color
    tya         ; Get the offset mask
    and ($10,x) ; X is already 0 from the loop above
    pla
    ora ($10,x) ; Apply the color.
    sta ($10,x) ; And store it.

    ; We're done!
    rts

; y: 0001_1111
; x: 0001_1111
;           ^^ bits
; shift y << 3
; shift x >> 2
