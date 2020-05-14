use crate::cpu_6502::test_helpers::*;

/// These tests assert the various operations the CPU can do. They use a high-level
/// API based off of macros to tersely assert the behavior.
/// For instance this command will run the test:
///
/// `cargo test cpu_6502::test::immediate_mode::adc1`
///
///      TestName Register Status  Program
///             |     |     |      |
///             v     v     v      v
/// register_a!(adc1, 0x33, P, "lda #$22\nadc #$11",);

/// Test all of the immedate mode instructions.
#[rustfmt::skip]
mod immediate_mode {
  use super::*;

  mod adc_basics {
    use super::*;
    // This first test shows: 0x22 + 0x11 == 0x33.
    // P is the default "P" or status register values.
    register_a!(adc1, 0x33, P, "
      lda #$22
      adc #$11
    ");
    // This add doesn't do anything, but the N, or negative flag is set since the most
    // significant bit is 1.
    register_a!(adc2, 0xff, P | N, "
      lda #$FF
      adc #$00
    ");
    // Here we overflow the u8.
    register_a!(
      adc3,
      0x00,
      P
      | C // For unsigned numbers, the carry bit is flipped, since the result carries over.
      | Z, // The result is 0x00 (with the carry only in the status register)
      "
        lda #$FF  ; 255 signed, or -1 unsigned
        adc #$01  ;   1 signed, or 1 unsigned
      "
    );
    // This is a similar result as above, but the final resut is not 0.
    register_a!(adc4, 0x01, P | C, "lda #$FF\nadc #$02");
    // Check that this uses the carry flag.
    register_a!(adc_carry, 0x34, P, "
      sec      ; Set the carry flag
      lda #$11 ; Load A with a value
      adc #$22 ; This should add all three values
              ; = 0x01 + 0x11 + 0x22
    ");
  }

  mod adc_overflow_carry {
    // This section tests the adc cases from:
    // http://www.6502.org/tutorials/vflag.html
    use super::*;
    register_a!(test_1_1, 0x02, P, "
      CLC      ; 1 + 1 = 2, returns C = 0
      LDA #$01 ;            returns V = 0
      ADC #$01
    ");
    // 0b0000_0001
    // 0x1111_1111
    // 1_0000_0000
    register_a!(test_1_neg1, 0x00, P | C | Z, "
      CLC      ; 1 + -1 = 0, returns C = 1
      LDA #$01 ;                     V = 0
      ADC #$FF
    ");

    // 0b0111_1111
    // 0b0000_0001
    // 0b1000_0000
    register_a!(test_127_1, 0b1000_0000, P | V | N, "
      CLC      ; 127 + 1 = 128, returns C = 0
      LDA #$7F ;                        V = 1
      ADC #$01
    ");

    // 0x80 + 0xff
    // 0b1000_0000
    // 0b1111_1111
    // 1_0111_1111
    register_a!(neg128_negative_1, 0b0111_1111, P | C | V, "
      CLC      ; -128 + -1 = -129, returns C = 1
      LDA #$80 ;                           V = 1
      ADC #$FF
    ");

    // 0b0011_1111  a
    // 0b0100_0000  operand
    // 0b0000_0001  carry
    // 0b0000_0000  result
    register_a!(carry, 0b1000_0000, P | V | N, "
      SEC      ; Note: SEC, not CLC
      LDA #$3F ; 63 + 64 + 1 = 128, returns V = 1
      ADC #$40
    ");
  }

  mod sbc_overflow_carry {
    // This section tests the sbc cases from:
    // http://www.6502.org/tutorials/vflag.html
    use super::*;
    // 0b0000_0000   two's comp   0b0000_0000
    // 0b0000_0001       ->       0b1111_1111
    //                            0b1111_1111
    register_a!(test_0_minus_1, negative(1), P | N, "
      SEC      ; 0 - 1 = -1, returns V = 0
      LDA #$00
      SBC #$01
    ");

    // 0b1000_0000    0b1000_0000
    // 0b0000_0001 -> 0b1111_1111
    //              0b1_0111_1111
    register_a!(neg128_minus_1, negative(129), P | C | V, "
      SEC      ; -128 - 1 = -129, returns V = 1
      LDA #$80
      SBC #$01
    ");

    // 0b0111_1111    0b0111_1111
    // 0b1111_1111 -> 0b0000_0001
    //                0b1000_0000
    register_a!(test_127_minus_neg1, 128, P | V | N, "
      SEC      ; 127 - -1 = 128, returns V = 1
      LDA #$7F
      SBC #$FF
    ");

    //   0b1100_0000    0b1100_0000
    // - 0b0100_0000 => 0b1011_1111
    //                  1_0111_1111
    register_a!(clc, negative(129), P | C | V, "
      CLC      ; Note: CLC, not SEC
      LDA #$C0 ; -64 - 64 - 1 = -129, returns V = 1
      SBC #$40
    ");
  }

  mod compare {
    use super::*;
    // http://6502.org/tutorials/compare_instructions.html
    status!(cmp_lt, P | N,     "lda #$11\ncmp #$22");
    status!(cmp_gt, P | C,     "lda #$22\ncmp #$11");
    status!(cmp_eq, P | C | Z, "lda #$11\ncmp #$11");
    status!(cpx_lt, P | N,     "ldx #$11\ncpx #$22");
    status!(cpx_gt, P | C,     "ldx #$22\ncpx #$11");
    status!(cpx_eq, P | C | Z, "ldx #$11\ncpx #$11");
    status!(cpy_lt, P | N,     "ldy #$11\ncpy #$22");
    status!(cpy_gt, P | C,     "ldy #$22\ncpy #$11");
    status!(cpy_eq, P | C | Z, "ldy #$11\ncpy #$11");
  }

  register_a!(and, 0b1010_0000, P | N, "
    lda #%11110000
    and #%10101010
  ");
  register_a!(eor, 0b0101_1010, P, "
    lda #%11110000
    eor #%10101010
  ");
  register_a!(ora, 0b1111_1010, P | N, "
    lda #%11110000
    ora #%10101010
  ");

  register_a!(lda, 0x22, P, "lda #$22");
  register_x!(ldx, 0x22, P, "ldx #$22");
  register_y!(ldy, 0x22, P, "ldy #$22");

  register_a!(nop, 0x00, P, "nop #$22");

  register_a!(sbc1, 0x22,        P | C, "
    sec       ; Always set the carry flag first.
    lda #$33
    sbc #$11
  ");
  register_a!(sbc2, 0x00,        P | Z | C, "
    sec       ; Always set the carry flag first.
    lda #$33
    sbc #$33
  ");
  register_a!(sbc3, negative(1), P | N, "
    sec       ; Always set the carry flag first.
    lda #$33
    sbc #$34
  ");

  mod illegal {
    // register_a!(, "alr #$22", 0x22, P);
    // register_a!(, "anc #$22", 0x22, P);
    // register_a!(, "axs #$22", 0x22, P);
    // register_a!(, "arr #$22", 0x22, P);
    // register_a!(, "lax #$22", 0x22, P);
    // register_a!(, "xaa #$22", 0x22, P);
  }
}

// Run two's complement on a u8.
fn negative(n: u8) -> u8 {
  !n + 1
}
