; Immediate values use the values provided immediately
; after the instruction
immediate:
  lda #$22 ; Load the value 0x22 into register a

; Implied values need no operands.
implied:
  sec ; Set the clear bit.

; Relative addresses are only used for branching operations.
; They move the current pc 127 bytes forward, or 128 bytes
; based on the signed byte provided.
relative:
  bcc relative
