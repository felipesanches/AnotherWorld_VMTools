BALL_IMAGE      EQU 0x031E ; This is the address of one of the code wheel symbols.
PADDLE_IMAGE    EQU 0x0604
CINEMATIC EQU 1

; These are the VM vars we'll use:
BALL_X               EQU 0x00
BALL_Y               EQU 0x01
BALL_ZOOM            EQU 0x02
PADDLE_ZOOM          EQU 0x03
BALL_VX              EQU 0x05
BALL_VY              EQU 0x06
PADDLE_X             EQU 0x07
PADDLE_Y             EQU 0x08
ZOOM_SPEED           EQU 0x09
RANDOM_SEED          EQU 0x3C
HACK_VAR_54          EQU 0x54
HACK_VAR_67          EQU 0x67
LAST_KEYCHAR         EQU 0xDA
HACK_VAR_DC          EQU 0xDC
HERO_POS_UP_DOWN     EQU 0xE5
MUS_MARK             EQU 0xF4
HACK_VAR_F7          EQU 0xF7
SCROLL_Y             EQU 0xF9
HERO_ACTION          EQU 0xFA
HERO_POS_JUMP_DOWN   EQU 0xFB
HERO_POS_LEFT_RIGHT  EQU 0xFC
HERO_POS_MASK        EQU 0xFD
HERO_ACTION_POS_MASK EQU 0xFE
PAUSE_SLICES         EQU 0xFF

; constants:
PADDLE_SPEED         EQU 4
BALL_SPEED           EQU 1

; This programs uses a single VM thread and a single videopage.

boot:

      call var_init
      call video_init

mainloop:
      call draw_scene
      call update_ball_position
      call update_paddle_position
      call detect_border_colision
      break ; breaking is necessary prior to each sample of user input
      jmp mainloop

update_ball_position:
      ; update the screen coordinates based on the current x and y velocities:
      add [BALL_X], [BALL_VX]
      add [BALL_Y], [BALL_VY]
      ret

show_screen_1:
      load id=0x0012
      ret

show_screen_2:
      load id=0x0013
      ret

RIGHT_ARROW EQU 1
LEFT_ARROW  EQU 2
DOWN_ARROW  EQU 4
UP_ARROW    EQU 8

update_paddle_position:
;maybe_move_right:
      jne [HERO_ACTION_POS_MASK], RIGHT_ARROW, maybe_move_left
      add [PADDLE_X], PADDLE_SPEED
maybe_move_left:
      jne [HERO_ACTION_POS_MASK], LEFT_ARROW, maybe_zoom_in
      sub [PADDLE_X], PADDLE_SPEED
maybe_zoom_in:
      jne [HERO_ACTION_POS_MASK], UP_ARROW, maybe_zoom_out
      add [PADDLE_ZOOM], [ZOOM_SPEED]
      call show_screen_1
maybe_zoom_out:
      jne [HERO_ACTION_POS_MASK], DOWN_ARROW, dont_move_paddle
      sub [PADDLE_ZOOM], [ZOOM_SPEED]
      call show_screen_2
dont_move_paddle:
      ret

detect_border_colision:
      ; mirror the velocities if the symbol collides with the borders:
      jl [BALL_X], 320, _1
      mov [BALL_VX], 0
      sub [BALL_VX], BALL_SPEED
_1:
      jg [BALL_X], 0, _2
      mov [BALL_VX], BALL_SPEED
_2:
      jl [BALL_Y], 200, _3
      mov [BALL_VY], 0
      sub [BALL_VY], BALL_SPEED
_3:
      jg [BALL_Y], 0, _4
      mov [BALL_VY], BALL_SPEED
_4:
      ret

var_init:
      ; The VX and VY vars are strictly positive values, so we treat
      ; zero as -SPEED,
      ; SPEED as zero
      ; 2*SPEED as +SPEED
      mov [PAUSE_SLICES], 2 ; 2*20ms = 40ms per frame = 25 frames / sec
      mov [ZOOM_SPEED], 1
      mov [BALL_X], 160
      mov [BALL_Y], 100
      mov [PADDLE_X], 40
      mov [PADDLE_Y], 180
      mov [BALL_ZOOM], 0x40
      mov [PADDLE_ZOOM], 0x40
      mov [BALL_VX], BALL_SPEED
      mov [BALL_VY], BALL_SPEED
      ret

video_init:
      setPalette 0x03 ; This is one the color palette used for that code wheel symbol.
      selectVideoPage 0x00
      ret

draw_scene:
      ; fill the whole page with a background color:
      fill page=0x00, color=0x07

      ; draw the symbol:
      video type=CINEMATIC, offset=BALL_IMAGE, x=[BALL_X], y=[BALL_Y], zoom=0x40
      video type=CINEMATIC, offset=PADDLE_IMAGE, x=[PADDLE_X], y=[PADDLE_Y], zoom=[PADDLE_ZOOM]

      ; and update the screen:
      blitFramebuffer 0x00
      ret
