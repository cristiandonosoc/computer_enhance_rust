; ========================================================================
; LISTING 52
; ========================================================================

bits 16

mov dx, 6
mov bp, 1000

mov si, 0
init_loop_start:
	mov word [bp + si], si
	add si, 2
	cmp si, dx
	jnz init_loop_start

mov bx, 0
mov si, 0
add_loop_start:
	mov cx, word [bp + si]
	add bx, cx
	add si, 2
	cmp si, dx
	jnz add_loop_start


; ANSWER
; bx: 0x0006
; cx: 0x0004
; dx: 0x0006
; bp: 0x03e8
; si: 0x0006
; ip: 0x0023
; flags: PZ
