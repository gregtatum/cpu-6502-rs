main:
  jsr compare_when_equal_true
  jsr compare_when_equal_false
  jsr compare_x_when_equal_true
  jsr compare_x_when_equal_false
  ; There is also compare y, but it's pretty much the same as compare x.
  kil

success:
  rts

failure:
  kil

compare_when_equal_true:
  lda #$22
  sta $00
  cmp $00
  beq success
  jmp failure

compare_when_equal_false:
  lda #$22
  sta $00
  lda #$33
  cmp $00
  beq failure
  jmp success

compare_x_when_equal_true:
  lda #$22
  sta $00
  tax
  lda #$00 ; Clear out A for clarity
  cpx $00
  beq success
  jmp failure

compare_x_when_equal_false:
  lda #$22
  sta $00
  lda #$33
  tax
  lda #$00 ; Clear out A for clarity
  cpx $00
  beq failure
  jmp success
