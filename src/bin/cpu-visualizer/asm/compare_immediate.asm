; This tests comparisons for immediate values.

main:
  jsr compare_when_equal_true
  jsr compare_when_equal_false
  kil

success:
  rts

failure:
  kil

compare_when_equal_true:
  lda #$22
  cmp #$22
  beq success
  jmp failure

compare_when_equal_false:
  lda #$22
  cmp #$33
  beq failure
  jmp success
