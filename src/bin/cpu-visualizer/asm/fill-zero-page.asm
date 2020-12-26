; Fill the zero page with values
; Load up the zero page.
dosomemath:
  lda #$22
  sta $12   ; 0x10 + 0x02
  ; Load up the registers
  lda #$11
  ldx #$02
  ; Do the math
  clc
  adc $10,x
