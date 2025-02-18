include "hardware.inc"

GAMESTATE_MENU = 0
GAMESTATE_INPUT_MENU = 1
GAMESTATE_MAP = 2
GAMESTATE_SERIAL = 3

SERIAL_CONNECTION_STATE_INTERNAL = 2
SERIAL_CONNECTION_STATE_EXTERNAL = 1
SERIAL_CONNECTION_STATE_UNCONNECTED = -1

SERIAL_DATA_SYNC_FLAG = $DF

LCDC_DEFAULT = LCDCF_BGON | LCDCF_OBJON | LCDCF_OBJ8 | LCDCF_BG9800 | LCDCF_BG8800 | LCDCF_WINOFF | LCDCF_WIN9C00 | LCDCF_ON
