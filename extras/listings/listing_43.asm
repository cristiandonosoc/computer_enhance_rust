; ========================================================================
; LISTING 43
; ========================================================================

bits 16

mov ax, 1
mov bx, 2
mov cx, 3
mov dx, 4

mov sp, 5
mov bp, 6
mov si, 7
mov di, 8

; ANSWER
; ax: 0x0001
; bx: 0x0002
; cx: 0x0003
; dx: 0x0004
; sp: 0x0005
; bp: 0x0006
; si: 0x0007
; di: 0x0008
; ip: 0x0018
