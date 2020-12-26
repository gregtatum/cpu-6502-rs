; This file demonstrates setting and clearing processor flags.

zero_out_status_register:
      ; nv__dIzc - Default state
  cli ; nv__dizc - make every flag 0

trivially_set_flags:
  ; The decimal, interrupt, and carry flags can be controlled
  ; directly through setting and clearing flags.
  sed ; nv__Dizc - set decimal
  sei ; nv__DIzc - set interrupt
  sec ; nv__DIzC - set carry
trivially_clear_flags:
  clc ; nv__DIzc - clear carry
  cli ; nv__Dizc - clear interrupt
  cld ; nv__DIzc - clear decimal

carry_flags:
  ; Test carry.
  lda #$ff  ; nv__dizc - Set register A to just before it carries.
  adc #$10  ; nv__dizC - This will overflow the 8th bit, setting the carry flag.

reset_carry:
  lda #$00 ; nv__dizC - The C is still set from the last operation
  adc #$10 ; nv__dizc - Performing another add that doesn't overflow
           ;            will clear the carry flag.

overflow_signed_numbers:
  lda #%01111111 ; nv__dizc - 127 is the largest signed integer.
  adc #$00000001 ; NV__dizc - Addding one more bit overflows, flipping the sign.
  clv            ; Nv__DIzc - clear overflow
  lda #0         ; nv__DIzc - Make the A register non-negative
