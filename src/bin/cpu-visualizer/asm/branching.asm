; This file tests the branching operations. The first branch
; should be skipped, and the second should be taken.

test_branch_if_plus:
  lda #250            ; Set a negative number.
  bpl branch_if_minus ; This will be ignored since it's a negative.
  lda #50             ; Set a positive number.
  bpl branch_if_minus ; This will pass and skip the NOPs.
  nop
  nop
  nop

branch_if_minus:
  lda #50                      ; Set a positive number.
  bmi branch_if_overflow_clear ; This will be ignored since it's a negative.
  lda #250                     ; Set a negative number.
  bmi branch_if_overflow_clear ; This will pass and skip the NOPs.
  nop
  nop
  nop

branch_if_overflow_clear:
  clc
  lda #%01111111
  adc #$00000001             ; Overflow the positive to negative number.
  bvc branch_if_overflow_set ; Skip!
  clv                        ; Clear the overflow bit
  bvc branch_if_overflow_set ; Pass!
  nop
  nop
  nop

branch_if_overflow_set:
  clv                  ; Clear the overflow bit
  bvs branch_carry_set ; Skip!
  clc
  lda #%01111111       ;
  adc #$00000001       ; Overflow the positive to negative number.
  bvs branch_carry_set ; Pass!
  nop
  nop
  nop

branch_carry_set:
  clc
  bcs branch_carry_clear
  sec
  bcs branch_carry_clear
  nop
  nop
  nop

branch_carry_clear:
  sec
  bcc branch_not_equal
  clc
  bcc branch_not_equal
  nop
  nop
  nop

; branch on Z=0
branch_not_equal:
  lda #$00
  bne branch_if_equal
  lda #$01
  bne branch_if_equal
  nop
  nop
  nop

; branch on Z=1
branch_if_equal:
  lda #$01
  beq end
  lda #$00
  beq end
  nop
  nop
  nop

end:
  nop
  nop
  nop
