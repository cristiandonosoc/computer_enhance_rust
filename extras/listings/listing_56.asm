; ========================================================================
; LISTING 56
; ========================================================================

bits 16

mov bx, 1000
mov bp, 2000
mov si, 3000
mov di, 4000

mov cx, bx
mov dx, 12

mov dx, [1000]

mov cx, [bx]
mov cx, [bp]
mov [si], cx
mov [di], cx

mov cx, [bx + 1000]
mov cx, [bp + 1000]
mov [si + 1000], cx
mov [di + 1000], cx

add cx, dx
add [di + 1000], cx
add dx, 50

; ANSWER
; bx: 0x03e8
; dx: 0x0032
; bp: 0x07d0
; si: 0x0bb8
; di: 0x0fa0
; ip: 0x0037
; cycles: 192
