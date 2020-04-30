
Label:                          ; A label and a comment
        lda     #$20            ; A 6502 instruction plus comment
L1:     ldx     #$20            ; Same with label
L2:     .byte   "Hello world"   ; Label plus control command
        mymac   $20             ; Macro expansion
        MySym = 3*L1            ; Symbol definition
MaSym   = Label                 ; Another symbol
