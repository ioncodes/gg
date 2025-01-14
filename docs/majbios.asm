;
; Game Gear (Majesco) BIOS
; Dumped by Mike Gordon on the 04/22/2001
; Disassembled and commented by Omar Cornut on the 04/22/2001
; Version 1
;

00000000: F3            DI                      ; Disable interrupts

00000001: 21 00 00      LD HL,0000h             ; Set paging registers
00000004: 22 FC FF      LD (FFFCh),HL           ; -> Disable SRAM
00000007: 21 01 02      LD HL,0201h             ; -> 0:0, 1:1, 2:2
0000000A: 22 FE FF      LD (FFFEh),HL           ;

0000000D: 21 00 C0      LD HL,C000h             ; Clear RAM
00000010: 11 01 C0      LD DE,C001h             ; from C000h to DFF0h
00000013: 01 EF 1F      LD BC,1FEFh             ;
00000016: 75            LD (HL),L               ;
00000017: ED B0         LDIR                    ;

00000019: 31 F0 DF      LD SP,DFF0h             ; Init stack pointer @ DFF0h

0000001C: 3E F5         LD A,F5h                ; Gear-to-gear stuffs
0000001E: D3 02         OUTA (02h)              ; ..
00000020: DB 02         INA (02h)               ; ..
00000022: FE F5         CP F5h                  ; <to comment>
00000024: 20 0B         JR NZ,+0Bh              ;
00000026: 3E FA         LD A,FAh                ;
00000028: D3 02         OUTA (02h)              ;
0000002A: DB 02         INA (02h)               ;
0000002C: FE FA         CP FAh                  ;
0000002E: 20 01         JR NZ,+01h              ;
00000030: AF            XOR A                   ; Copy a detected configuration 
00000031: 32 01 C7      LD (C701h),A            ; thing to RAM[701h]
00000034: 3E FF         LD A,FFh                ; Later called 'Gear-to-Gear xx option'
00000036: D3 02         OUTA (02h)              ;

00000038: DB 7E         INA (7Eh)               ; Wait for Vertical Blank
0000003A: FE B0         CP B0h                  ;
0000003C: 20 FA         JR NZ,-06h              ;

0000003E: 11 00 7F      LD DE,7F00h             ; Set VDP address to 3F00h (&Sprite_0_Y)
00000041: CD 22 01      CALL VDP_Set_Addr (0122h) then write D0h to disable
00000044: 3E D0         LD A,D0h                ; sprites. 
00000046: D3 BE         OUTA (BEh)              ; 

00000048: 21 E9 03      LD HL,03E9h             ; Init VDP Registers
0000004B: 01 BF 16      LD BC,16BFh             ; Stored at 03E9h
0000004E: ED B3         OTIR                    ;

00000050: 21 00 C0      LD HL,C000h             ; 
00000053: 11 00 78      LD DE,7800h             ; Set VDP address to 3800h (&Tile_256)
00000056: CD 22 01      CALL VDP_Set_Addr (0122h)
00000059: 0E BE         LD C,BEh                ;
0000005B: ED B3         OTIR                    ; Copy 22*6 == 132 bytes from
0000005D: ED B3         OTIR                    ; the beginning of RAM to
0000005F: ED B3         OTIR                    ; Video ram ??
00000061: ED B3         OTIR                    ;
00000063: 18 03         JR +03h ДДДДДДДї        ; -> (A)
00000065: 00            NOP            і
                                       і
-- NMI                                 і
00000066: ED 45         RETN           і        ; Return from NMI (unused)
                                       і
00000068: ED B3         OTIR <ДДДДДДДДДЩ        ; (A)
0000006A: ED B3         OTIR                    ;

0000006C: 21 00 C0      LD HL,C000h             ; Write palette memory,
0000006F: 11 00 C0      LD DE,C000h             ; with source being C000h
00000072: 01 BE 40      LD BC,40BEh             ; (beginning of RAM)
00000075: CD 22 01      CALL VDP_Set_Addr (0122h) ??
00000078: ED B3         OTIR                    ;

0000007A: 21 6A 01      LD HL,016Ah             ;
0000007D: 06 04         LD B,04h                ;
0000007F: 3A 01 C7      LD A,(C701h)            ; Load Gear-to-Gear ?? option
00000082: B7            OR A                    ; Depending on it, copy
00000083: 20 03         JR NZ,+03h              ; source will be from 016Ah
00000085: 21 66 01      LD HL,0166h             ; or 0166h
00000088: CD 22 01      CALL VDP_Set_Addr (0122h)
0000008B: ED B3         OTIR                    ;

0000008D: 21 DC 02      LD HL,02DCh             ;
00000090: 3A 01 C7      LD A,(C701h)            ;
00000093: B7            OR A                    ;
00000094: 20 03         JR NZ,+03h              ;
00000096: 21 6C 01      LD HL,016Ch             ;
00000099: 11 00 40      LD DE,4000h             ;
0000009C: CD 29 01	CALL 0129h

0000009F: CD FA 00      CALL Check_TMR_Sega (00FAh) ; Check for "TMR SEGA" string
000000A2: 3A 00 C7      LD A,(C700h)            ; Read result (stored there)
000000A5: B7            OR A                    ; If it's zero, goes to an infinite loop
                                                ; Else, it should be a 'A'
000000A6: 28 FE         JR Z,-02h               ; *CRACK* Write 00,00 to A6h to bypass the check
000000A8: 21 AA 03      LD HL,03AAh             
000000AB: 3A 01 C7	LD A,(C701h)
000000AE: B7		OR A
000000AF: 20 03		JR NZ,+03h
000000B1: 21 6B 03	LD HL,036Bh
000000B4: 11 4C 7A	LD DE,7A4Ch
000000B7: 06 15		LD B,15h
000000B9: CD 51 01	CALL 0151h
000000BC: 11 CC 7A	LD DE,7ACCh
000000BF: 06 15		LD B,15h
000000C1: CD 51 01	CALL 0151h
000000C4: 11 4C 7B	LD DE,7B4Ch
000000C7: 06 15		LD B,15h
000000C9: CD 51 01	CALL 0151h

000000CC: 11 E0 81      LD DE,81E0h             ; VDP_Reg[1] = E0h
000000CF: CD 22 01      CALL VDP_Set_Reg (0122h); Enable VBL Interrupt

000000D2: 06 07         LD B,07h                ; Big loop, catched by the
000000D4: 21 00 00      LD HL,0000h <ДДї        ; vertical blanking interrupt
000000D7: 2B            DEC HL <ДДДДї  і        ; *CRACK* Write 01 at 0xD6 to disable startup delay
000000D8: 7D            LD A,L      і  і        ; It is supposed to be a delay?
000000D9: B4            OR H        і  і        ; It *might* be really needed
000000DA: 20 FB         JR NZ,-05h ДЩ  і        ;
000000DC: 10 F9         DJNZ -07h ДДДДДЩ        ;

000000DE: 11 A0 81      LD DE,81A0h             ; VDP_Reg[1] = A0h
000000E1: CD 22 01      CALL VDP_Set_Reg (0122h); Disable VBL Interrupt

000000E4: 21 F2 00      LD HL,00F2h             ; Copy 8 bytes of code
000000E7: 11 00 C8      LD DE,C800h             ; from 0xF2 to RAM[0x800]
000000EA: 01 08 00      LD BC,0008h             ; (this is the mapping code)
000000ED: ED B0         LDIR                    ; ..
000000EF: C3 00 C8      JP C800h                ; then jump there!

--------
; This function is copied and run, starting at RAM[800h]
-- Function: Cartridge_Map
000000F2: 3E A8         LD A,A8h                ; Write A8h to C000h
000000F4: 32 00 C0      LD (C000h),A            ; Read back from C000h
000000F7: D3 3E         OUTA (3Eh)              ; And stick that to port 3Eh
000000F9: C7            RST 00h                 ; Reset.. start game
; Note from Zoop:
; This is where the game start
; Starting condition: VBL interrupts are disabled
; and VDP registers are as defined at 03E9h

(00FAh)
-- Function: Check_TMR_Sega
000000FA: CD 0E 01      CALL 010Eh              ; Try to read at 1FF0h
000000FD: CD 09 01      CALL 0109h              ; Else at 3FF0h
00000100: CD 04 01      CALL 0104h              ; Else at 7FF0h
00000103: C9            RET                     ; If any is good, RAM[700h] will be set
00000104: 11 F0 1F       LD DE,1FF0h                      ; de = 1FF0
00000107: 18 08          JR +08h ДДДДДДДДДДДДДДДДДДДДДДДї ; 
00000109: 11 F0 3F       LD DE,3FF0h                    і ; de = 3FF0
0000010C: 18 03          JR +03h ДДДДДДДДДДДДДДДДДДДДДДДґ ;
0000010E: 11 F0 7F       LD DE,7FF0h                    і ; de = 7FF0
00000111: 21 5E 01       LD HL,CONST_TMR_SEGA (015Eh) <ДЩ ; hl = "TMR SEGA"
00000114: 06 08          LD B,08h                         ; b = strlen(hl)
00000116: 1A             LD A,(DE) <ДДДДДї                ;
00000117: BE             CP (HL)         і                ; Compare strings
00000118: C0             RET NZ          і                ; and return if
00000119: 23             INC HL          і                ; they are differents
0000011A: 13             INC DE          і                ; (-> not "TMR_SEGA")
0000011B: 10 F9          DJNZ -07h ДДДДДДЩ
0000011D: 32 00 C7       LD (C700h),A                     ; If found, write 'A' to RAM[700h] ('A' is the last char of "TMR_SEGA")
00000120: F1            POP AF                            ; and return to the caller of FAh
00000121: C9            RET                               ; by manually poping from the stack
--

(0122h)
-- Function: VDP_Set_Addr
-- Function: VDP_Set_Reg
-- In: DE
-- Destroy: A
00000122: 7B		LD A,E
00000123: D3 BF		OUTA (BFh)
00000125: 7A		LD A,D
00000126: D3 BF		OUTA (BFh)
00000128: C9		RET
--

(0129h)
-- Function: xxx
00000129: 06 04		LD B,04h
0000012B: C5		PUSH BC
0000012C: D5             PUSH DE
0000012D: 7E              LD A,(HL)
0000012E: 23              INC HL
0000012F: B7              OR A
00000130: 28 19           JR Z,+19h
00000132: CB BF           RES 7,A
00000134: 47              LD B,A
00000135: CD 22 01        CALL VDP_Set_AddrReg (0122h)
00000138: 7E              LD A,(HL)
00000139: D3 BE           OUTA (BEh)
0000013B: F2 3F 01        JP P,013Fh ДДї
0000013E: 23              INC HL       і
0000013F: 13              INC DE <ДДДДДЩ
00000140: 13              INC DE
00000141: 13              INC DE
00000142: 13              INC DE
00000143: 10 F0           DJNZ -10h
00000145: FA 2D 01        JP M,012Dh
00000148: 23              INC HL
00000149: 18 E2           JR -1Eh
0000014B: D1             POP DE
0000014C: 13             INC DE
0000014D: C1		POP BC
0000014E: 10 DB		DJNZ -25h
00000150: C9		RET
--

(0151h)
-- Function:
00000151: CD 22 01      CALL VDP_Set_AddrReg (0122h)
00000154: AF		XOR A
00000155: 0E BE         LD C,BEh <ДДДї
00000157: ED A3         OUTI         і
00000159: ED 79         OUT (C),A    і
0000015B: 20 F8         JR NZ,-08h ДДЩ
0000015D: C9		RET
--

(015Eh)
[DATA: CONST_TMR_SEGA]
"TMR SEGA"
[/DATA]

(xxxxh)
[DATA: ??]
00, 0F, EE, 0E
[/DATA]

(xxxxh)
[DATA: ??]
30, 3F, 09, 00
[/DATA]

0000016E: 81		ADD C
0000016F: 0F		RRCA
00000170: 03		INC BC
00000171: 0C		INC C
00000172: 95		SUB L
00000173: 0F		RRCA
00000174: 0C		INC C
00000175: 0C		INC C
00000176: 00		NOP
00000177: 8F		ADC A
00000178: CC CC CF	CALL Z,CFCCh
0000017B: 8C		ADC H
0000017C: 0C		INC C
0000017D: 0C		INC C
0000017E: 00		NOP
0000017F: 87		ADD A
00000180: CC CC 8C	CALL Z,8CCCh
00000183: CC CC C7	CALL Z,C7CCh
00000186: 00		NOP
00000187: 8F		ADC A
00000188: 05		DEC B
00000189: CC 83 8F	CALL Z,8F83h
0000018C: 00		NOP
0000018D: 8C		ADC H
0000018E: 05		DEC B
0000018F: CC 83 87	CALL Z,8783h
00000192: 00		NOP
00000193: C7		RST 00h
00000194: 05		DEC B
00000195: CC 8B 87	CALL Z,878Bh
00000198: 00		NOP
00000199: 8F		ADC A
0000019A: CC CC 0F	CALL Z,0FCCh
0000019D: CC CC 8F	CALL Z,8FCCh
000001A0: 00		NOP
000001A1: 9F		SBC A
000001A2: 05		DEC B
000001A3: 19		ADD HL,DE
000001A4: 83		ADD E
000001A5: 9F		SBC A
000001A6: 00		NOP
000001A7: 01 05 81	LD BC,8105h
000001AA: 8A		ADC D
000001AB: 01 00 F1	LD BC,F100h
000001AE: 99		SBC C
000001AF: 99		SBC C
000001B0: F0		RET P
000001B1: 98		SBC B
000001B2: 98		SBC B
000001B3: F0		RET P
000001B4: 00		NOP
000001B5: 03		INC BC
000001B6: 98		SBC B
000001B7: 81		ADD C
000001B8: F0		RET P
000001B9: 03		INC BC
000001BA: 60		LD H,B
000001BB: 82		ADD D
000001BC: 00		NOP
000001BD: 0F		RRCA
000001BE: 05		DEC B
000001BF: 19		ADD HL,DE
000001C0: 8E		ADC (HL)
000001C1: 0F		RRCA
000001C2: 00		NOP
000001C3: 1F		RRA
000001C4: 99		SBC C
000001C5: 99		SBC C
000001C6: 9F		SBC A
000001C7: 99		SBC C
000001C8: 99		SBC C
000001C9: 19		ADD HL,DE
000001CA: 00		NOP
000001CB: 00		NOP
000001CC: 80		ADD B
000001CD: 80		ADD B
000001CE: 00		NOP
000001CF: 03		INC BC
000001D0: 80		ADD B
000001D1: 81		ADD C
000001D2: 00		NOP
000001D3: 06 CC		LD B,CCh
000001D5: 8B		ADC E
000001D6: 78		LD A,B
000001D7: 00		NOP
000001D8: 8C		ADC H
000001D9: CC EC FC	CALL Z,FCECh
000001DC: FC DC CC	CALL M,CCDCh
000001DF: 00		NOP
000001E0: F8		RET M
000001E1: 05		DEC B
000001E2: CC 8E F8	CALL Z,F88Eh
000001E5: 00		NOP
000001E6: F9		LD SP,HL
000001E7: C1		POP BC
000001E8: C1		POP BC
000001E9: F1		POP AF
000001EA: C1		POP BC
000001EB: C1		POP BC
000001EC: F9		LD SP,HL
000001ED: 00		NOP
000001EE: F0		RET P
000001EF: 98		SBC B
000001F0: 98		SBC B
000001F1: F0		RET P
000001F2: 03		INC BC
000001F3: 98		SBC B
000001F4: 81		ADD C
000001F5: 00		NOP
000001F6: 06 18		LD B,18h
000001F8: 83		ADD E
000001F9: 1F		RRA
000001FA: 00		NOP
000001FB: 31 05 33	LD SP,3305h
000001FE: AE		XOR (HL)
000001FF: 31 00 E3	LD SP,E300h
00000202: 33		INC SP
00000203: 33		INC SP
00000204: 03		INC BC
00000205: 33		INC SP
00000206: 33		INC SP
00000207: E3		EX HL,(SP)
00000208: 00		NOP
00000209: E4 06 07	CALL PO,0706h
0000020C: C7		RST 00h
0000020D: 06 06		LD B,06h
0000020F: E6 00		AND 00h
00000211: 63		LD H,E
00000212: 66		LD H,(HL)
00000213: 66		LD H,(HL)
00000214: E3		EX HL,(SP)
00000215: E0		RET PO
00000216: 66		LD H,(HL)
00000217: 23		INC HL
00000218: 00		NOP
00000219: C7		RST 00h
0000021A: 66		LD H,(HL)
0000021B: 06 C7		LD B,C7h
0000021D: 66		LD H,(HL)
0000021E: 66		LD H,(HL)
0000021F: C7		RST 00h
00000220: 00		NOP
00000221: C0		RET NZ
00000222: 00		NOP
00000223: 00		NOP
00000224: 80		ADD B
00000225: 00		NOP
00000226: 00		NOP
00000227: C0		RET NZ
00000228: 00		NOP
00000229: F9		LD SP,HL
0000022A: C1		POP BC
0000022B: C1		POP BC
0000022C: F1		POP AF
0000022D: 03		INC BC
0000022E: C1		POP BC
0000022F: 8A		ADC D
00000230: 00		NOP
00000231: F0		RET P
00000232: 99		SBC C
00000233: 99		SBC C
00000234: F1		POP AF
00000235: 99		SBC C
00000236: 99		SBC C
00000237: 98		SBC B
00000238: 00		NOP
00000239: F1		POP AF
0000023A: 05		DEC B
0000023B: 99		SBC C
0000023C: BB		CP E
0000023D: F1		POP AF
0000023E: 00		NOP
0000023F: 04		INC B
00000240: 8C		ADC H
00000241: DC FC AC	CALL C,ACFCh
00000244: 8C		ADC H
00000245: 8C		ADC H
00000246: 00		NOP
00000247: 78		LD A,B
00000248: CC C0 78	CALL Z,78C0h
0000024B: 0C		INC C
0000024C: CC 78 00	CALL Z,0078h
0000024F: F8		RET M
00000250: C1		POP BC
00000251: C1		POP BC
00000252: F1		POP AF
00000253: C1		POP BC
00000254: C1		POP BC
00000255: F8		RET M
00000256: 00		NOP
00000257: F0		RET P
00000258: 98		SBC B
00000259: 98		SBC B
0000025A: 80		ADD B
0000025B: B9		CP C
0000025C: 99		SBC C
0000025D: F9		LD SP,HL
0000025E: 00		NOP
0000025F: 60		LD H,B
00000260: 60		LD H,B
00000261: F0		RET P
00000262: 90		SUB B
00000263: F8		RET M
00000264: 98		SBC B
00000265: 98		SBC B
00000266: 00		NOP
00000267: 1F		RRA
00000268: 18 18		JR +18h
0000026A: 1E 18		LD E,18h
0000026C: 18 1F		JR +1Fh
0000026E: 00		NOP
0000026F: 23		INC HL
00000270: 33		INC SP
00000271: 3B		DEC SP
00000272: 3F		CCF
00000273: 37		SCF
00000274: 33		INC SP
00000275: 31 00 3F	LD SP,3F00h
00000278: 06 0C		LD B,0Ch
0000027A: 8D		ADC L
0000027B: 00		NOP
0000027C: 3E 30		LD A,30h
0000027E: 30 3C		JR NC,+3Ch
00000280: 30 30		JR NC,+30h
00000282: 3E 00		LD A,00h
00000284: 7C		LD A,H
00000285: 66		LD H,(HL)
00000286: 66		LD H,(HL)
00000287: 7C		LD A,H
00000288: 03		INC BC
00000289: 66		LD H,(HL)
0000028A: 82		ADD D
0000028B: 00		NOP
0000028C: 7C		LD A,H
0000028D: 03		INC BC
0000028E: 66		LD H,(HL)
0000028F: 9D		SBC L
00000290: 7C		LD A,H
00000291: 60		LD H,B
00000292: 60		LD H,B
00000293: 00		NOP
00000294: 63		LD H,E
00000295: 66		LD H,(HL)
00000296: 66		LD H,(HL)
00000297: 63		LD H,E
00000298: 60		LD H,B
00000299: 66		LD H,(HL)
0000029A: 63		LD H,E
0000029B: 00		NOP
0000029C: C7		RST 00h
0000029D: 0C		INC C
0000029E: 0C		INC C
0000029F: 87		ADD A
000002A0: 00		NOP
000002A1: 0C		INC C
000002A2: C7		RST 00h
000002A3: 00		NOP
000002A4: 80		ADD B
000002A5: C0		RET NZ
000002A6: 00		NOP
000002A7: 80		ADD B
000002A8: C0		RET NZ
000002A9: C0		RET NZ
000002AA: 80		ADD B
000002AB: 00		NOP
000002AC: C1		POP BC
000002AD: 05		DEC B
000002AE: C0		RET NZ
000002AF: 83		ADD E
000002B0: FC 00 F9	CALL M,F900h
000002B3: 06 61		LD B,61h
000002B5: 82		ADD D
000002B6: 00		NOP
000002B7: F0		RET P
000002B8: 05		DEC B
000002B9: 98		SBC B
000002BA: 81		ADD C
000002BB: F0		RET P
000002BC: 06 00		LD B,00h
000002BE: 02		LD (BC),A
000002BF: C0		RET NZ
000002C0: 00		NOP
000002C1: 7F		LD A,A
000002C2: 00		NOP
000002C3: 7F		LD A,A
000002C4: 00		NOP
000002C5: 7F		LD A,A
000002C6: 00		NOP
000002C7: 03		INC BC
000002C8: 00		NOP
000002C9: 00		NOP
000002CA: 7F		LD A,A
000002CB: 00		NOP
000002CC: 7F		LD A,A
000002CD: 00		NOP
000002CE: 7F		LD A,A
000002CF: 00		NOP
000002D0: 03		INC BC
000002D1: 00		NOP
000002D2: 00		NOP
000002D3: 7F		LD A,A
000002D4: 00		NOP
000002D5: 7F		LD A,A
000002D6: 00		NOP
000002D7: 7F		LD A,A
000002D8: 00		NOP
000002D9: 03		INC BC
000002DA: 00		NOP
000002DB: 00		NOP
000002DC: 08		EX AF,AF'
000002DD: 00		NOP
000002DE: 81		ADD C
000002DF: 7C		LD A,H
000002E0: 03		INC BC
000002E1: 66		LD H,(HL)
000002E2: 81		ADD C
000002E3: 7C		LD A,H
000002E4: 03		INC BC
000002E5: 60		LD H,B
000002E6: 81		ADD C
000002E7: 7C		LD A,H
000002E8: 03		INC BC
000002E9: 66		LD H,(HL)
000002EA: 81		ADD C
000002EB: 7C		LD A,H
000002EC: 03		INC BC
000002ED: 66		LD H,(HL)
000002EE: 81		ADD C
000002EF: 3C		INC A
000002F0: 06 66		LD B,66h
000002F2: 82		ADD D
000002F3: 3C		INC A
000002F4: 7C		LD A,H
000002F5: 06 66		LD B,66h
000002F7: 81		ADD C
000002F8: 7C		LD A,H
000002F9: 07		RLCA
000002FA: 66		LD H,(HL)
000002FB: 02		LD (BC),A
000002FC: 3C		INC A
000002FD: 02		LD (BC),A
000002FE: 66		LD H,(HL)
000002FF: 02		LD (BC),A
00000300: 60		LD H,B
00000301: 02		LD (BC),A
00000302: 66		LD H,(HL)
00000303: 85		ADD L
00000304: 3C		INC A
00000305: 7E		LD A,(HL)
00000306: 60		LD H,B
00000307: 60		LD H,B
00000308: 7C		LD A,H
00000309: 03		INC BC
0000030A: 60		LD H,B
0000030B: 85		ADD L
0000030C: 7E		LD A,(HL)
0000030D: 7C		LD A,H
0000030E: 66		LD H,(HL)
0000030F: 66		LD H,(HL)
00000310: 7C		LD A,H
00000311: 03		INC BC
00000312: 66		LD H,(HL)
00000313: 81		ADD C
00000314: 7C		LD A,H
00000315: 04		INC B
00000316: 66		LD H,(HL)
00000317: 81		ADD C
00000318: 3C		INC A
00000319: 03		INC BC
0000031A: 18 88		JR -78h
0000031C: 46		LD B,(HL)
0000031D: 66		LD H,(HL)
0000031E: 76		HALT
0000031F: 7E		LD A,(HL)
00000320: 7E		LD A,(HL)
00000321: 6E		LD L,(HL)
00000322: 66		LD H,(HL)
00000323: 62		LD H,D
00000324: 07		RLCA
00000325: 60		LD H,B
00000326: 82		ADD D
00000327: 7E		LD A,(HL)
00000328: 3C		INC A
00000329: 06 18		LD B,18h
0000032B: 02		LD (BC),A
0000032C: 3C		INC A
0000032D: 8B		ADC E
0000032E: 66		LD H,(HL)
0000032F: 60		LD H,B
00000330: 7C		LD A,H
00000331: 3E 06		LD A,06h
00000333: 66		LD H,(HL)
00000334: 3C		INC A
00000335: 7E		LD A,(HL)
00000336: 60		LD H,B
00000337: 60		LD H,B
00000338: 7C		LD A,H
00000339: 04		INC B
0000033A: 60		LD H,B
0000033B: 85		ADD L
0000033C: 41		LD B,C
0000033D: 63		LD H,E
0000033E: 77		LD (HL),A
0000033F: 7F		LD A,A
00000340: 6B		LD L,E
00000341: 03		INC BC
00000342: 63		LD H,E
00000343: 8D		ADC L
00000344: 3C		INC A
00000345: 66		LD H,(HL)
00000346: 66		LD H,(HL)
00000347: 60		LD H,B
00000348: 6E		LD L,(HL)
00000349: 66		LD H,(HL)
0000034A: 66		LD H,(HL)
0000034B: 3E 3C		LD A,3Ch
0000034D: 24		INC H
0000034E: 66		LD H,(HL)
0000034F: 66		LD H,(HL)
00000350: 7E		LD A,(HL)
00000351: 03		INC BC
00000352: 66		LD H,(HL)
00000353: 81		ADD C
00000354: 7E		LD A,(HL)
00000355: 07		RLCA
00000356: 18 06		JR +06h
00000358: 00		NOP
00000359: 02		LD (BC),A
0000035A: 18 00		JR +00h
0000035C: 7F		LD A,A
0000035D: 00		NOP
0000035E: 21 00 00	LD HL,0000h
00000361: 7F		LD A,A
00000362: 00		NOP
00000363: 21 00 00	LD HL,0000h
00000366: 7F		LD A,A
00000367: 00		NOP
00000368: 21 00 00	LD HL,0000h
0000036B: 00		NOP
0000036C: 00		NOP
0000036D: 00		NOP
0000036E: 01 02 03	LD BC,0302h
00000371: 04		INC B
00000372: 05		DEC B
00000373: 06 07		LD B,07h
00000375: 08		EX AF,AF'
00000376: 09		ADD HL,BC
00000377: 0A		LD A,(BC)
00000378: 0B		DEC BC
00000379: 0C		INC C
0000037A: 0D		DEC C
0000037B: 0E 00		LD C,00h
0000037D: 00		NOP
0000037E: 00		NOP
0000037F: 00		NOP
00000380: 00		NOP
00000381: 00		NOP
00000382: 0F		RRCA
00000383: 10 11		DJNZ +11h
00000385: 12		LD (DE),A
00000386: 13		INC DE
00000387: 14		INC D
00000388: 15		DEC D
00000389: 16 17		LD D,17h
0000038B: 18 19		JR +19h
0000038D: 1A		LD A,(DE)
0000038E: 1B		DEC DE
0000038F: 1C		INC E
00000390: 1D		DEC E
00000391: 1E 00		LD E,00h
00000393: 00		NOP
00000394: 00		NOP
00000395: 00		NOP
00000396: 1F		RRA
00000397: 20 21		JR NZ,+21h
00000399: 22 23 24	LD (2423h),HL
0000039C: 25		DEC H
0000039D: 26 27		LD H,27h
0000039F: 28 27		JR Z,+27h
000003A1: 29		ADD HL,HL
000003A2: 19		ADD HL,DE
000003A3: 2A 2B 2C	LD HL,(2C2Bh)
000003A6: 2D		DEC L
000003A7: 2E 2F		LD L,2Fh
000003A9: 00		NOP
000003AA: 00		NOP
000003AB: 00		NOP
000003AC: 00		NOP
000003AD: 01 02 03	LD BC,0302h
000003B0: 04		INC B
000003B1: 05		DEC B
000003B2: 06 07		LD B,07h
000003B4: 04		INC B
000003B5: 00		NOP
000003B6: 08		EX AF,AF'
000003B7: 09		ADD HL,BC
000003B8: 00		NOP
000003B9: 03		INC BC
000003BA: 02		LD (BC),A
000003BB: 00		NOP
000003BC: 00		NOP
000003BD: 00		NOP
000003BE: 00		NOP
000003BF: 00		NOP
000003C0: 05		DEC B
000003C1: 0A		LD A,(BC)
000003C2: 04		INC B
000003C3: 07		RLCA
000003C4: 02		LD (BC),A
000003C5: 00		NOP
000003C6: 0B		DEC BC
000003C7: 0C		INC C
000003C8: 06 07		LD B,07h
000003CA: 0A		LD A,(BC)
000003CB: 0D		DEC C
000003CC: 07		RLCA
000003CD: 00		NOP
000003CE: 0E 02		LD C,02h
000003D0: 03		INC BC
000003D1: 0F		RRCA
000003D2: 00		NOP
000003D3: 00		NOP
000003D4: 0D		DEC C
000003D5: 07		RLCA
000003D6: 10 11		DJNZ +11h
000003D8: 00		NOP
000003D9: 07		RLCA
000003DA: 0A		LD A,(BC)
000003DB: 12		LD (DE),A
000003DC: 07		RLCA
000003DD: 02		LD (BC),A
000003DE: 01 02 0C	LD BC,0C02h
000003E1: 0D		DEC C
000003E2: 07		RLCA
000003E3: 0D		DEC C
000003E4: 00		NOP
000003E5: 0B		DEC BC
000003E6: 12		LD (DE),A
000003E7: 04		INC B
000003E8: 13		INC DE

(03E9h)
[DATA: VDP Registers]
 16h, 80h       ;       VDP_Reg[0] = 16h        Configuration 1
 A0h, 81h       ;       VDP_Reg[1] = A0h        Configuration 2
 FFh, 82h       ;       VDP_Reg[2] = FFh        ;
 FFh, 83h       ;       VDP_Reg[3] = FFh        ;
 FFh, 84h       ;       VDP_Reg[4] = FFh        ;
 FFh, 85h       ;       VDP_Reg[5] = FFh        ;
 FFh, 86h       ;       VDP_Reg[6] = FFh        ;
 00h, 87h       ;       VDP_Reg[7] = 00h        Border
 00h, 88h       ;       VDP_Reg[8] = 00h        Horizontal Scrolling
 00h, 89h       ;       VDP_Reg[9] = 00h        Vertical Scrolling
 FFh, 8Ah       ;      VDP_Reg[10] = FFh        Line counter
[/DATA]

000003FF: FF            RST 38h