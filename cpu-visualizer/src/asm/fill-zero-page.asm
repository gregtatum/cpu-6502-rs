; Fill the zero page with incrementing vlues.
lda #$22
root:
  sta $00,x
  adc #1
  inx
  jmp root
