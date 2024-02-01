/*
Type: Mapped
Platform: Z80
Architecture: Z80

Segments:
r-x  0x00000000-0x00000400 
---  0x00000400-0x0000040a 

Sections:
0x00000400-0x0000040a  .synthetic_builtins  {External}
*/

00000000  int16_t entry()

void* const __return_addr  {Frame offset 0}

00000000  f3         DI
00000001  210000     LD HL,0
00000004  22fcff     LD (0xfffc),HL  {0x0}
00000007  210102     LD HL,0x201
0000000a  22feff     LD (0xfffe),HL  {data_201}
0000000d  2100c0     LD HL,0xc000
00000010  1101c0     LD DE,0xc001
00000013  01ef1f     LD BC,0x1fef
00000016  75         LD (HL),L  {entry}  {0xc000}
00000017  edb0       LDIR
00000019  31f0df     LD SP,0xdff0
0000001c  3ef5       LD A,0xf5
0000001e  d302       OUT (0x0002),A
00000020  db02       IN A,(0x0002)
00000022  fef5       CP 0xf5
00000024  200b       JR nz,0x0031  {0x0}

00000026  // Does some sort of verification. Not sure what this does but it does
00000026  // seem to expect the same value back as it writes.
00000026  3efa       LD A,0xfa
00000028  d302       OUT (0x0002),A
0000002a  db02       IN A,(0x0002)
0000002c  fefa       CP 0xfa
0000002e  2001       JR nz,0x0031  {0x0}

00000030  af         XOR A  {0x0}

00000031  3201c7     LD (0xc701),A
00000034  3eff       LD A,0xff
00000036  d302       OUT (0x0002),A

00000038  db7e       IN A,(0x007e)
0000003a  feb0       CP 0xb0
0000003c  20fa       JR nz,0x0038  {0x1}

0000003e  11007f     LD DE,0x7f00
00000041  cd2201     CALL vdp_set_address_register
00000044  3ed0       LD A,0xd0  // disable sprites
00000046  d3be       OUT (0x00be),A
00000048  21e903     LD HL,0x3e9
0000004b  01bf16     LD BC,0x16bf
0000004e  edb3       OTIR
00000050  2100c0     LD HL,0xc000
00000053  110078     LD DE,0x7800
00000056  cd2201     CALL vdp_set_address_register
00000059  0ebe       LD C,0xbe
0000005b  edb3       OTIR
0000005d  edb3       OTIR
0000005f  edb3       OTIR
00000061  edb3       OTIR
00000063  1803       JR 0x0068

00000065                 00                                                                                     .

00000066  int16_t sub_66()

void* const __return_addr  {Frame offset 0}

00000066  ed45       RETN


{ Continuation of function entry }

00000068  edb3       OTIR
0000006a  edb3       OTIR
0000006c  2100c0     LD HL,0xc000
0000006f  1100c0     LD DE,0xc000
00000072  01be40     LD BC,0x40be
00000075  cd2201     CALL vdp_set_address_register
00000078  edb3       OTIR
0000007a  216a01     LD HL,0x16a
0000007d  0604       LD B,4
0000007f  3a01c7     LD A,(0xc701)
00000082  b7         OR A
00000083  2003       JR nz,0x0088


{ Continuation of function sub_66 }

00000068  edb3       OTIR
0000006a  edb3       OTIR
0000006c  2100c0     LD HL,0xc000
0000006f  1100c0     LD DE,0xc000
00000072  01be40     LD BC,0x40be
00000075  cd2201     CALL vdp_set_address_register
00000078  edb3       OTIR
0000007a  216a01     LD HL,0x16a
0000007d  0604       LD B,4
0000007f  3a01c7     LD A,(0xc701)
00000082  b7         OR A
00000083  2003       JR nz,0x0088


{ Continuation of function entry }

00000085  216601     LD HL,0x166


{ Continuation of function sub_66 }

00000085  216601     LD HL,0x166


{ Continuation of function entry }

00000088  cd2201     CALL vdp_set_address_register
0000008b  edb3       OTIR
0000008d  21dc02     LD HL,0x2dc
00000090  3a01c7     LD A,(0xc701)
00000093  b7         OR A
00000094  2003       JR nz,0x0099


{ Continuation of function sub_66 }

00000088  cd2201     CALL vdp_set_address_register
0000008b  edb3       OTIR
0000008d  21dc02     LD HL,0x2dc
00000090  3a01c7     LD A,(0xc701)
00000093  b7         OR A
00000094  2003       JR nz,0x0099


{ Continuation of function entry }

00000096  216c01     LD HL,0x16c


{ Continuation of function sub_66 }

00000096  216c01     LD HL,0x16c


{ Continuation of function entry }

00000099  110040     LD DE,0x4000
0000009c  cd2901     CALL load_sega_license_message
0000009f  cdfa00     CALL verify_sega_string_in_cartridge
000000a2  3a00c7     LD A,(0xc700)
000000a5  b7         OR A


{ Continuation of function sub_66 }

00000099  110040     LD DE,0x4000
0000009c  cd2901     CALL load_sega_license_message
0000009f  cdfa00     CALL verify_sega_string_in_cartridge
000000a2  3a00c7     LD A,(0xc700)
000000a5  b7         OR A


{ Continuation of function entry }

000000a6  // According to my emulator this endlessly loops if the string could
000000a6  // not be verified correctly.
000000a6  28fe       JR z,0x00a6


{ Continuation of function sub_66 }

000000a6  28fe       JR z,0x00a6


{ Continuation of function entry }

000000a8  21aa03     LD HL,0x3aa
000000ab  3a01c7     LD A,(0xc701)
000000ae  b7         OR A
000000af  2003       JR nz,0x00b4


{ Continuation of function sub_66 }

000000a8  21aa03     LD HL,0x3aa
000000ab  3a01c7     LD A,(0xc701)
000000ae  b7         OR A
000000af  2003       JR nz,0x00b4


{ Continuation of function entry }

000000b1  216b03     LD HL,0x36b


{ Continuation of function sub_66 }

000000b1  216b03     LD HL,0x36b


{ Continuation of function entry }

000000b4  114c7a     LD DE,0x7a4c
000000b7  0615       LD B,0x15
000000b9  cd5101     CALL vdp_set_tilemap_indices
000000bc  11cc7a     LD DE,0x7acc
000000bf  0615       LD B,0x15
000000c1  cd5101     CALL vdp_set_tilemap_indices
000000c4  114c7b     LD DE,0x7b4c
000000c7  0615       LD B,0x15
000000c9  cd5101     CALL vdp_set_tilemap_indices
000000cc  11e081     LD DE,0x81e0
000000cf  cd2201     CALL vdp_set_address_register
000000d2  0607       LD B,7
000000d4  210000     LD HL,0


{ Continuation of function sub_66 }

000000b4  114c7a     LD DE,0x7a4c
000000b7  0615       LD B,0x15
000000b9  cd5101     CALL vdp_set_tilemap_indices
000000bc  11cc7a     LD DE,0x7acc
000000bf  0615       LD B,0x15
000000c1  cd5101     CALL vdp_set_tilemap_indices
000000c4  114c7b     LD DE,0x7b4c
000000c7  0615       LD B,0x15
000000c9  cd5101     CALL vdp_set_tilemap_indices
000000cc  11e081     LD DE,0x81e0
000000cf  cd2201     CALL vdp_set_address_register
000000d2  0607       LD B,7
000000d4  210000     LD HL,0


{ Continuation of function entry }

000000d7  2b         DEC HL
000000d8  7d         LD A,L
000000d9  b4         OR H
000000da  20fb       JR nz,0x00d7


{ Continuation of function sub_66 }

000000d7  2b         DEC HL
000000d8  7d         LD A,L
000000d9  b4         OR H
000000da  20fb       JR nz,0x00d7


{ Continuation of function entry }

000000dc  10f9       DJNZ 0x00d7


{ Continuation of function sub_66 }

000000dc  10f9       DJNZ 0x00d7


{ Continuation of function entry }

000000de  // Copies the routine execute_cartridge to RAM at 0xc800 and then
000000de  // jumps to it.
000000de  11a081     LD DE,0x81a0
000000e1  cd2201     CALL vdp_set_address_register
000000e4  21f200     LD HL,execute_cartridge
000000e7  1100c8     LD DE,0xc800
000000ea  010800     LD BC,8
000000ed  edb0       LDIR
000000ef  c300c8     JP 0xc800


{ Continuation of function sub_66 }

000000de  11a081     LD DE,0x81a0
000000e1  cd2201     CALL vdp_set_address_register
000000e4  21f200     LD HL,0xf2
000000e7  1100c8     LD DE,0xc800
000000ea  010800     LD BC,8
000000ed  edb0       LDIR
000000ef  c300c8     JP 0xc800


// Disable BIOS, enables Cartridge. Resets system.
000000f2  int16_t execute_cartridge()

void* const __return_addr  {Frame offset 0}

000000f2  3ea8       LD A,0xa8
000000f4  3200c0     LD (0xc000),A  {0xa8}
000000f7  d33e       OUT (0x003e),A
000000f9  c7         RST 0  {verify_sega_string_in_cartridge}
{ Falls through into verify_sega_string_in_cartridge }


000000fa  int16_t verify_sega_string_in_cartridge()

void* const __return_addr  {Frame offset 0}

000000fa  cd0e01     CALL verify_sega_string_at_7ff0
000000fd  cd0901     CALL verify_sega_string_at_3ff0
00000100  cd0401     CALL verify_sega_string_at_1ff0
00000103  c9         RET {__return_addr}


00000104  int16_t verify_sega_string_at_1ff0()

void* const __return_addr  {Frame offset 0}
int16_t arg_2  {Frame offset 2}

00000104  11f01f     LD DE,0x1ff0
00000107  1808       JR 0x0111


00000109  int16_t verify_sega_string_at_3ff0()

void* const __return_addr  {Frame offset 0}
int16_t arg_2  {Frame offset 2}

00000109  11f03f     LD DE,0x3ff0
0000010c  1803       JR 0x0111


0000010e  int16_t verify_sega_string_at_7ff0()

void* const __return_addr  {Frame offset 0}
int16_t arg_2  {Frame offset 2}

0000010e  11f07f     LD DE,0x7ff0
00000111  215e01     LD HL,0x15e  {"TMR SEGA"}
00000114  0608       LD B,8


{ Continuation of function verify_sega_string_at_1ff0 }

00000111  215e01     LD HL,SEGA_TRADEMARK  {"TMR SEGA"}
00000114  0608       LD B,8


{ Continuation of function verify_sega_string_at_3ff0 }

00000111  215e01     LD HL,0x15e  {"TMR SEGA"}
00000114  0608       LD B,8


{ Continuation of function verify_sega_string_at_1ff0 }

00000116  1a         LD A,(DE)
00000117  be         CP (HL)
00000118  c0         RET nz {__return_addr}
00000119  23         INC HL
0000011a  13         INC DE
0000011b  10f9       DJNZ 0x0116


{ Continuation of function verify_sega_string_at_3ff0 }

00000116  1a         LD A,(DE)
00000117  be         CP (HL)
00000118  c0         RET nz {__return_addr}
00000119  23         INC HL
0000011a  13         INC DE
0000011b  10f9       DJNZ 0x0116


{ Continuation of function verify_sega_string_at_7ff0 }

00000116  1a         LD A,(DE)
00000117  be         CP (HL)
00000118  c0         RET nz {__return_addr}
00000119  23         INC HL
0000011a  13         INC DE
0000011b  10f9       DJNZ 0x0116


{ Continuation of function verify_sega_string_at_1ff0 }

0000011d  3200c7     LD (0xc700),A
00000120  f1         POP AF {__return_addr}
00000121  c9         RET {arg_2}


{ Continuation of function verify_sega_string_at_3ff0 }

0000011d  3200c7     LD (0xc700),A
00000120  f1         POP AF {__return_addr}
00000121  c9         RET {arg_2}


{ Continuation of function verify_sega_string_at_7ff0 }

0000011d  3200c7     LD (0xc700),A
00000120  f1         POP AF {__return_addr}
00000121  c9         RET {arg_2}


00000122  int16_t vdp_set_address_register(int16_t arg1 @ DE)

void* const __return_addr  {Frame offset 0}
int16_t arg1  {Register DE}

00000122  7b         LD A,E
00000123  d3bf       OUT (0x00bf),A
00000125  7a         LD A,D
00000126  d3bf       OUT (0x00bf),A
00000128  c9         RET {__return_addr}


00000129  int16_t load_sega_license_message(int16_t vram_addr @ DE, char* rom_addr @ HL)

int16_t var_4  {Frame offset -4}
int16_t vram_addr_1  {Frame offset -4}
int16_t var_2  {Frame offset -2}
int16_t var_2_1  {Frame offset -2}
void* const __return_addr  {Frame offset 0}
int16_t vram_addr  {Register DE}
char* rom_addr  {Register HL}

00000129  0604       LD B,4

0000012b  c5         PUSH BC {var_2_1}
0000012c  d5         PUSH DE {vram_addr_1}

0000012d  7e         LD A,(HL)
0000012e  23         INC HL
0000012f  b7         OR A
00000130  2819       JR z,0x014b

00000132  cbbf       RES 7,A
00000134  47         LD B,A

00000135  cd2201     CALL vdp_set_address_register
00000138  7e         LD A,(HL)
00000139  d3be       OUT (0x00be),A
0000013b  f23f01     JP p,0x013f

0000013e  23         INC HL

0000013f  13         INC DE
00000140  13         INC DE
00000141  13         INC DE
00000142  13         INC DE
00000143  10f0       DJNZ 0x0135

00000145  fa2d01     JP m,0x012d

00000148  23         INC HL
00000149  18e2       JR 0x012d

0000014b  d1         POP DE {vram_addr_1}
0000014c  13         INC DE
0000014d  c1         POP BC {var_2_1}
0000014e  10db       DJNZ 0x012b

00000150  c9         RET {__return_addr}


// Copy tile indices to VRAM (as required by the nametable address). The tiles
// (tilemap) are loaded in the right order already, which allows the BIOS to
// write the indices (as 2 byte sequences) from 0 -> N by increasing A on each
// loop to VRAM.

00000151  int16_t vdp_set_tilemap_indices()

void* const __return_addr  {Frame offset 0}

00000151  cd2201     CALL vdp_set_address_register
00000154  af         XOR A  {0x0}

00000155  0ebe       LD C,0xbe
00000157  eda3       OUTI
00000159  ed79       OUT (C),A
0000015b  20f8       JR nz,0x0155  {0x0}

0000015d  c9         RET {__return_addr}

0000015e  char const SEGA_TRADEMARK[0x9] = "TMR SEGA", 0

00000167                       0f ee 0e                                                                           ...
0000016a  data_16a:
0000016a                                30 3f                                                                        0?
0000016c  data_16c:
0000016c                                      09 00 81 0f 03 0c 95 0f 0c 0c 00 8f cc cc cf 8c 0c 0c 00 87              ....................
00000180  cc cc 8c cc cc c7 00 8f 05 cc 83 8f 00 8c 05 cc 83 87 00 c7 05 cc 8b 87 00 8f cc cc 0f cc cc 8f  ................................
000001a0  00 9f 05 19 83 9f 00 01 05 81 8a 01 00 f1 99 99 f0 98 98 f0 00 03 98 81 f0 03 60 82 00 0f 05 19  ..........................`.....
000001c0  8e 0f 00 1f 99 99 9f 99 99 19 00 00 80 80 00 03 80 81 00 06 cc 8b 78 00 8c cc ec fc fc dc cc 00  ......................x.........
000001e0  f8 05 cc 8e f8 00 f9 c1 c1 f1 c1 c1 f9 00 f0 98 98 f0 03 98 81 00 06 18 83 1f 00 31 05 33 ae 31  ...........................1.3.1
00000200  00                                                                                               .
00000201  data_201:
00000201     e3 33 33 03 33 33 e3 00 e4 06 07 c7 06 06 e6 00 63 66 66 e3 e0 66 23 00 c7 66 06 c7 66 66 c7   .33.33..........cff..f#..f..ff.
00000220  00 c0 00 00 80 00 00 c0 00 f9 c1 c1 f1 03 c1 8a 00 f0 99 99 f1 99 99 98 00 f1 05 99 bb f1 00 04  ................................
00000240  8c dc fc ac 8c 8c 00 78 cc c0 78 0c cc 78 00 f8 c1 c1 f1 c1 c1 f8 00 f0 98 98 80 b9 99 f9 00 60  .......x..x..x.................`
00000260  60 f0 90 f8 98 98 00 1f 18 18 1e 18 18 1f 00 23 33 3b 3f 37 33 31 00 3f 06 0c 8d 00 3e 30 30 3c  `..............#3;?731.?....>00<
00000280  30 30 3e 00 7c 66 66 7c 03 66 82 00 7c 03 66 9d 7c 60 60 00 63 66 66 63 60 66 63 00 c7 0c 0c 87  00>.|ff|.f..|.f.|``.cffc`fc.....
000002a0  00 0c c7 00 80 c0 00 80 c0 c0 80 00 c1 05 c0 83 fc 00 f9 06 61 82 00 f0 05 98 81 f0 06 00 02 c0  ....................a...........
000002c0  00 7f 00 7f 00 7f 00 03 00 00 7f 00 7f 00 7f 00 03 00 00 7f 00 7f 00 7f 00 03 00 00              ............................
000002dc  data_2dc:
000002dc                                                                                      08 00 81 7c                              ...|
000002e0  03 66 81 7c 03 60 81 7c 03 66 81 7c 03 66 81 3c 06 66 82 3c 7c 06 66 81 7c 07 66 02 3c 02 66 02  .f.|.`.|.f.|.f.<.f.<|.f.|.f.<.f.
00000300  60 02                                                                                            `.
00000302  data_302:
00000302        66 85 3c 7e 60 60 7c 03 60 85 7e 7c 66 66 7c 03 66 81 7c 04 66 81 3c 03 18 88                f.<~``|.`.~|ff|.f.|.f.<...

0000031c  int16_t sub_31c(int16_t arg1 @ DE, char* arg2 @ HL)

void* const __return_addr  {Frame offset 0}
int16_t arg1  {Register DE}
char* arg2  {Register HL}

0000031c  46         LD B,(HL)
0000031d  66         LD H,(HL)
0000031e  76         HALT
0000031f  7e         LD A,(HL)
00000320  7e         LD A,(HL)
00000321  6e         LD L,(HL)
00000322  66         LD H,(HL)
00000323  62         LD H,D
00000324  07         RLCA
00000325  60         LD H,B
00000326  82         ADD A,D
00000327  7e         LD A,(HL)
00000328  3c         INC A
00000329  0618       LD B,0x18
0000032b  02         LD (BC),A
0000032c  3c         INC A
0000032d  8b         ADC A,E
0000032e  66         LD H,(HL)
0000032f  60         LD H,B  {0x18}
00000330  7c         LD A,H  {0x18}
00000331  3e06       LD A,6
00000333  66         LD H,(HL)
00000334  3c         INC A  {0x7}
00000335  7e         LD A,(HL)
00000336  60         LD H,B  {0x18}
00000337  60         LD H,B
00000338  7c         LD A,H
00000339  04         INC B  {0x19}
0000033a  60         LD H,B  {0x19}
0000033b  85         ADD A,L  {0x18}
0000033c  41         LD B,C
0000033d  63         LD H,E
0000033e  77         LD (HL),A
0000033f  7f         LD A,A
00000340  6b         LD L,E
00000341  03         INC BC
00000342  63         LD H,E
00000343  8d         ADC A,L
00000344  3c         INC A
00000345  66         LD H,(HL)
00000346  66         LD H,(HL)
00000347  60         LD H,B
00000348  6e         LD L,(HL)
00000349  66         LD H,(HL)
0000034a  66         LD H,(HL)
0000034b  3e3c       LD A,0x3c
0000034d  24         INC H
0000034e  66         LD H,(HL)
0000034f  66         LD H,(HL)
00000350  7e         LD A,(HL)
00000351  03         INC BC
00000352  66         LD H,(HL)
00000353  81         ADD A,C
00000354  7e         LD A,(HL)
00000355  07         RLCA
00000356  1806       JR sub_35e

00000358                                                                          00                                               .

00000359  int16_t sub_359(int16_t arg1 @ AF, char* arg2 @ BC)

void* const __return_addr  {Frame offset 0}
int16_t arg1  {Register AF}
char* arg2  {Register BC}

00000359  02         LD (BC),A
0000035a  1800       JR 0x035c

0000035c  7f         LD A,A
0000035d  00         NOP  {sub_35e}
{ Falls through into sub_35e }


0000035e  int16_t sub_35e(int16_t arg1 @ AF, char* arg2 @ DE)

void* const __return_addr  {Frame offset 0}
int16_t arg1  {Register AF}
char* arg2  {Register DE}

0000035e  210000     LD HL,0
00000361  7f         LD A,A
00000362  00         NOP
00000363  210000     LD HL,0
00000366  7f         LD A,A
00000367  00         NOP
00000368  210000     LD HL,0
0000036b  00         NOP
0000036c  00         NOP
0000036d  00         NOP
0000036e  010203     LD BC,0x302
00000371  04         INC B
00000372  05         DEC B  {entry+3}
00000373  0607       LD B,7
00000375  08         EX AF,AF'
00000376  09         ADD HL,BC  {0x0}  {0x702}
00000377  0a         LD A,(BC)  {0x702}
00000378  0b         DEC BC  {0x701}
00000379  0c         INC C
0000037a  0d         DEC C  {data_2}  {data_1}
0000037b  0e00       LD C,0
0000037d  00         NOP
0000037e  00         NOP
0000037f  00         NOP
00000380  00         NOP
00000381  00         NOP
00000382  0f         RRCA
00000383  1011       DJNZ 0x0396  {0x1}

00000385  12         LD (DE),A
00000386  13         INC DE
00000387  14         INC D
00000388  15         DEC D
00000389  1617       LD D,0x17
0000038b  1819       JR 0x03a6


0000038d  int16_t sub_38d(void* const arg1 @ BC, char* arg2 @ DE, void* arg3 @ HL, int16_t arg4 @ AF')

void* const __return_addr  {Frame offset 0}
void* const arg1  {Register BC}
char* arg2  {Register DE}
void* arg3  {Register HL}
int16_t arg4  {Register AF'}

0000038d  1a         LD A,(DE)
0000038e  1b         DEC DE
0000038f  1c         INC E
00000390  1d         DEC E
00000391  1e00       LD E,0
00000393  00         NOP
00000394  00         NOP
00000395  00         NOP
00000396  1f         RRA
00000397  2021       JR nz,0x03ba


{ Continuation of function sub_35e }

00000396  1f         RRA
00000397  2021       JR nz,0x03ba  {0x1}

00000399  222324     LD (0x2423),HL  {0x702}
0000039c  25         DEC H  {entry+7}  {data_6}
0000039d  2627       LD H,0x27
0000039f  2827       JR z,0x03c8  {0x0}


{ Continuation of function sub_38d }

00000399  222324     LD (0x2423),HL
0000039c  25         DEC H
0000039d  2627       LD H,0x27
0000039f  2827       JR z,0x03c8


{ Continuation of function sub_35e }

000003a1  29         ADD HL,HL
000003a2  19         ADD HL,DE
000003a3  2a2b2c     LD HL,(0x2c2b)


{ Continuation of function sub_38d }

000003a1  29         ADD HL,HL
000003a2  19         ADD HL,DE
000003a3  2a2b2c     LD HL,(0x2c2b)
000003a6  2d         DEC L
000003a7  2e2f       LD L,0x2f
000003a9  00         NOP
000003aa  00         NOP
000003ab  00         NOP
000003ac  00         NOP
000003ad  010203     LD BC,0x302
000003b0  04         INC B
000003b1  05         DEC B  {entry+3}
000003b2  0607       LD B,7
000003b4  04         INC B  {data_8}
000003b5  00         NOP
000003b6  08         EX AF,AF'
000003b7  09         ADD HL,BC
000003b8  00         NOP
000003b9  03         INC BC  {0x803}


{ Continuation of function sub_35e }

000003a6  2d         DEC L
000003a7  2e2f       LD L,0x2f
000003a9  00         NOP
000003aa  00         NOP
000003ab  00         NOP
000003ac  00         NOP
000003ad  010203     LD BC,0x302
000003b0  04         INC B
000003b1  05         DEC B  {entry+3}
000003b2  0607       LD B,7
000003b4  04         INC B  {data_8}
000003b5  00         NOP
000003b6  08         EX AF,AF'
000003b7  09         ADD HL,BC
000003b8  00         NOP
000003b9  03         INC BC  {0x803}

000003ba  02         LD (BC),A
000003bb  00         NOP
000003bc  00         NOP
000003bd  00         NOP
000003be  00         NOP
000003bf  00         NOP
000003c0  05         DEC B
000003c1  0a         LD A,(BC)
000003c2  04         INC B
000003c3  07         RLCA
000003c4  02         LD (BC),A
000003c5  00         NOP
000003c6  0b         DEC BC
000003c7  0c         INC C


{ Continuation of function sub_38d }

000003ba  02         LD (BC),A
000003bb  00         NOP
000003bc  00         NOP
000003bd  00         NOP
000003be  00         NOP
000003bf  00         NOP
000003c0  05         DEC B
000003c1  0a         LD A,(BC)
000003c2  04         INC B
000003c3  07         RLCA
000003c4  02         LD (BC),A
000003c5  00         NOP
000003c6  0b         DEC BC
000003c7  0c         INC C


{ Continuation of function sub_35e }

000003c8  0607       LD B,7
000003ca  0a         LD A,(BC)
000003cb  0d         DEC C
000003cc  07         RLCA
000003cd  00         NOP
000003ce  0e02       LD C,2
000003d0  03         INC BC  {0x703}
000003d1  0f         RRCA
000003d2  00         NOP
000003d3  00         NOP
000003d4  0d         DEC C  {0x2}
000003d5  07         RLCA
000003d6  1011       DJNZ 0x03e9  {0x1}


{ Continuation of function sub_38d }

000003c8  0607       LD B,7
000003ca  0a         LD A,(BC)
000003cb  0d         DEC C
000003cc  07         RLCA
000003cd  00         NOP
000003ce  0e02       LD C,2
000003d0  03         INC BC  {0x703}
000003d1  0f         RRCA
000003d2  00         NOP
000003d3  00         NOP
000003d4  0d         DEC C  {0x2}
000003d5  07         RLCA
000003d6  1011       DJNZ 0x03e9  {0x1}


{ Continuation of function sub_35e }

000003d8  00         NOP
000003d9  07         RLCA
000003da  0a         LD A,(BC)  {0x602}
000003db  12         LD (DE),A
000003dc  07         RLCA
000003dd  02         LD (BC),A  {0x602}
000003de  01020c     LD BC,0xc02
000003e1  0d         DEC C
000003e2  07         RLCA
000003e3  0d         DEC C  {entry}
000003e4  00         NOP
000003e5  0b         DEC BC  {0xbff}
000003e6  12         LD (DE),A
000003e7  04         INC B  {data_c}
000003e8  13         INC DE


{ Continuation of function sub_38d }

000003d8  00         NOP
000003d9  07         RLCA
000003da  0a         LD A,(BC)  {0x602}
000003db  12         LD (DE),A
000003dc  07         RLCA
000003dd  02         LD (BC),A  {0x602}
000003de  01020c     LD BC,0xc02
000003e1  0d         DEC C
000003e2  07         RLCA
000003e3  0d         DEC C  {entry}
000003e4  00         NOP
000003e5  0b         DEC BC  {0xbff}
000003e6  12         LD (DE),A
000003e7  04         INC B  {data_c}
000003e8  13         INC DE


{ Continuation of function sub_35e }

000003e9  1680       LD D,0x80
000003eb  a0         AND B
000003ec  81         ADD A,C
000003ed  ff         RST 0x38
000003ee  82         ADD A,D
000003ef  ff         RST 0x38
000003f0  83         ADD A,E
000003f1  ff         RST 0x38
000003f2  84         ADD A,H
000003f3  ff         RST 0x38
000003f4  85         ADD A,L
000003f5  ff         RST 0x38
000003f6  86         ADD A,(HL)
000003f7  00         NOP
000003f8  87         ADD A,A
000003f9  00         NOP
000003fa  88         ADC A,B
000003fb  00         NOP
000003fc  89         ADC A,C
000003fd  ff         RST 0x38
000003fe  8a         ADC A,D
000003ff  ff         RST 0x38
00000400  00         NOP
00000401  00         NOP
00000402  00         NOP
00000403  00         NOP
00000404  00         NOP
00000405  00         NOP
00000406  00         NOP
00000407  00         NOP
00000408  00         NOP
00000409  00         NOP
0000040a  ??         ??


{ Continuation of function sub_38d }

000003e9  1680       LD D,0x80
000003eb  a0         AND B
000003ec  81         ADD A,C
000003ed  ff         RST 0x38
000003ee  82         ADD A,D
000003ef  ff         RST 0x38
000003f0  83         ADD A,E
000003f1  ff         RST 0x38
000003f2  84         ADD A,H
000003f3  ff         RST 0x38
000003f4  85         ADD A,L
000003f5  ff         RST 0x38
000003f6  86         ADD A,(HL)
000003f7  00         NOP
000003f8  87         ADD A,A
000003f9  00         NOP
000003fa  88         ADC A,B
000003fb  00         NOP
000003fc  89         ADC A,C
000003fd  ff         RST 0x38
000003fe  8a         ADC A,D
000003ff  ff         RST 0x38
00000400  00         NOP
00000401  00         NOP
00000402  00         NOP
00000403  00         NOP
00000404  00         NOP
00000405  00         NOP
00000406  00         NOP
00000407  00         NOP
00000408  00         NOP
00000409  00         NOP
0000040a  ??         ??

