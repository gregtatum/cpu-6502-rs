sec        ; Set the carry flag
lda #$11   ; Load A with a value
adc #$22   ; This should add all three values
            ; = 0x01 + 0x11 + 0x22 = 0x34
clc        ; Clear the carry bit

adc #$01   ; Add to the A register
inx        ; Increase the value of X in the register
sta $00,x  ;

adc #$01   ; Add to the A register
inx        ; Increase the value of X in the register
sta $00,x  ;

adc #$01   ; Add to the A register
inx        ; Increase the value of X in the register
sta $00,x  ;

clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
clc        ;
