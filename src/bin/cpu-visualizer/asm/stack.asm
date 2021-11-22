; Demonstrate simple pushing and pulling, useful for viewing in
; the debugger.
working_with_stack:
    ; Push 3 values onto the stack
    lda #$11
    pha
    lda #$22
    pha
    lda #$33
    pha

    ; Reset A
    lda #$00

    pla ; Restore $33 to the A register
    pla ; Restore $22 to the A register
    pla ; Restore $11 to the A register

; The stack values can be referenced by transfering the stack pointer into
; the X register, and using `lda $0100,x`
working_with_stack_directly:
    ; Push 3 values onto the stack
    lda #$44
    pha
    lda #$55
    pha
    lda #$66
    pha

    lda #$00 ; Reset A for clarity.

    tsx ; Put the stack into X. The stack is pointing at the next available
        ; value which hasn't been written to yet.

    jsr store_stack_to_zero_page ; Store $66
    jsr store_stack_to_zero_page ; Store $55
    jsr store_stack_to_zero_page ; Store $44
    pla
    kil


store_stack_to_zero_page:
    inx          ; Point at next available stack value.
    lda $0100,x  ; Read from the stack.
    sta $0000,y  ; Store it to the zero page.
                 ; (ZeroPageY mode is not supported, so use full address)
    iny          ; Move forward in the zero page.
    rts          ; Return from the subroutine.
