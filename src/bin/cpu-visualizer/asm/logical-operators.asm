logical_operators_and:
  lda #%11110000
  and #%10101010
  sta $00,x
  inx

logical_operators_or:
  lda #%11110000
  ora #%10101010
  sta $00,x
  inx
