; ========================================================================
; LISTING 51
; ========================================================================

bits 16

mov word [1000], 1
mov word [1002], 2
mov word [1004], 3
mov word [1006], 4

mov bx, 1000
mov word [bx + 4], 10

mov bx, word [1000]
mov cx, word [1002]
mov dx, word [1004]
mov bp, word [1006]

; ANSWER
; bx: 0x0001
; cx: 0x0002
; dx: 0x000a
; bp: 0x0004
; ip: 0x0030
