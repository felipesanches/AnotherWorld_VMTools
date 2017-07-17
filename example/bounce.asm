BALL_IMAGE    EQU 0x031E ; This is the address of one of the code wheel symbols.

; These are the VM vars we'll use:
BALL_X        EQU 0x00
BALL_Y        EQU 0x01
BALL_ZOOM     EQU 0x02
SPEED         EQU 0x03
BALL_VX       EQU 0x04
BALL_VY       EQU 0x05
PAUSE_SLICES  EQU 0xFF

; This programs uses a single VM thread and a single videopage.
init:
      mov [PAUSE_SLICES], 2 ; 2*20ms = 40ms per frame = 25 frames / sec
      mov [SPEED], 1
      mov [BALL_X], 160
      mov [BALL_Y], 100
      mov [BALL_ZOOM], 0x40

; The VX and VY vars are strictly positive values, so we treat
; zero as -SPEED,
; SPEED as zero
; 2*SPEED as +SPEED
      mov [BALL_VX], [SPEED]
      add [BALL_VX], [SPEED]
      mov [BALL_VY], [SPEED]
      add [BALL_VY], [SPEED]
      setPalette 0x03 ; This is one the color palette used for that code wheel symbol.
      selectVideoPage 0x00

mainloop:
      ; fill the whole page with a background color:
      fill page=0x00, color=0x07

      ; draw the symbol:
      video offset=BALL_IMAGE, x=[BALL_X], y=[BALL_Y], zoom=[BALL_ZOOM]

      ; and update the screen:
      blitFramebuffer 0x00

      mov [0x00], [0x00]
      ; update the screen coordinates based on the current x and y velocities:
      sub [BALL_X], [SPEED]
      add [BALL_X], [BALL_VX]
      sub [BALL_Y], [SPEED]
      add [BALL_Y], [BALL_VY]

      ; mirror the velocities if the symbol collides with the borders:
      jl [BALL_X], 320, _1
      mov [BALL_VX], 0
_1:
      jg [BALL_X], 0, _2
      mov [BALL_VX], [SPEED]
      add [BALL_VX], [SPEED]
_2:
      jl [BALL_Y], 200, _3
      mov [BALL_VY], 0
_3:
      jg [BALL_Y], 0, _4
      mov [BALL_VY], [SPEED]
      add [BALL_VY], [SPEED]
_4:
      jmp mainloop
