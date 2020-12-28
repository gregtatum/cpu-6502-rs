; Fill the zero page with the fibonacci sequence

; Set the initial arguments
jsr init
jsr fibonacci
kil

init:
  ; Store the first 2 values of the fibonacci sequence.
  lda #$00
  sta $00
  lda #$01
  sta $01
  ; Set the X value, which will be used to offset into the zero page.
  ldx #$00
  rts

fibonacci:
  ; Add the two previous arguments together.
  lda $00,x
  inx
  adc $00,x
  inx
  bcs return ; Return if the carry is set.
  ; Store the result and prepare the X value for the next loop
  sta $00,x
  dex
  jmp fibonacci

return:
  rts
