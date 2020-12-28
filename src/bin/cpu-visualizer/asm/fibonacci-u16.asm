; Fill the zero page with the fibonacci sequence

; Set the initial arguments
jsr init
jsr fibonacci
kil

init:
  ; Store the first 2 values of the fibonacci sequence.

  ; Little endian u16: 0
  lda #$00
  sta $00
  lda #$00
  sta $01

  ; Little endian u16: 1
  lda #$01
  sta $02
  lda #$00
  sta $03

  ; Set the X value, which will be used to offset into the zero page.
  ldx #$00
  rts

fibonacci:
  clc;
  ; Add the LE bytes two previous arguments together.
  ; A     B     C
  ; LE BE LE BE
  ; ^^
  lda $00,x
  inx
  inx
  ; A     B     C
  ; LE BE LE BE
  ;       ^^
  adc $00,x

  ; Only remember this if the final computation doesn't overflow.
  pha

  dex

  ; Add the BE bytes of the previous two arguments together
  ; A     B     C
  ; LE BE LE BE LE
  ;    ^^
  lda $00,x
  inx
  inx

  ; A     B     C
  ; LE BE LE BE LE
  ;          ^^
  adc $00,x

  ; Return if the carry is set, which means we overflowed the u16.
  bcs return


  ; Swap the BE result to Y
  inx
  inx
  ; A     B     C
  ; LE BE LE BE LE BE
  ;                ^^
  sta $00,x

  pla
  dex
  ; A     B     C
  ; LE BE LE BE LE BE
  ;             ^^
  sta $00,x

  dex
  dex
  ; A     B     C
  ; LE BE LE BE LE BE
  ;       ^^
  jmp fibonacci

return:
  rts

; These are the expected results, as expressed as LE bytes.
; $00  $0000
; $02  $0100
; $04  $0100
; $08  $0200
; $0a  $0300
; $0c  $0500
; $0e  $0800
; $10  $0d00
; $12  $1500
; $14  $2200
; $18  $3700
; $1a  $5900
; $1c  $9000
; $1e  $e900
; $20  $7901
; $22  $6202
; $24  $db03
; $28  $3d06
; $2a  $180a
; $2c  $5510
; $2e  $6d1a
; $31  $c22a
; $32  $2f45
; $33  $f16f
; $34  $20b5
