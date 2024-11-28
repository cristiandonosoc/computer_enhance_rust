; ========================================================================
; LISTING 44
; ========================================================================

bits 16

mov ax, 1
mov bx, 2
mov cx, 3
mov dx, 4

mov sp, ax
mov bp, bx
mov si, cx
mov di, dx

mov dx, sp
mov cx, bp
mov bx, si
mov ax, di

; ANSWER
; ax: 0x0004
; bx: 0x0003
; cx: 0x0002
; dx: 0x0001
; sp: 0x0001
; bp: 0x0002
; si: 0x0003
; di: 0x0004
