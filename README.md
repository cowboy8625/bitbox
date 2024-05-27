# Bitbox

Bitbox is currently just a register based virtual machine.  The goal of the project is to have a target IR (intermediate representation) to compile your language to and then provide you with a VM and a way to target native code.

#### Header
| offset | size | purpose                                       |
|:-------|:----:|:---------------------------------------------:|
|  0x00  |  4   | 0x42, 0x42, 0x4f, 0x58 in ASCII these four bytes constitute the magic number.|
|  0x04  |  4   | How big the .data section is                  |
|  0x08  |  4   | Entry point into .text section                |
|  0x0C  |  52  | Not used section                              |

#### Insturctions Supported

|instruction |type| arg1 | arg2 | arg3 |
|:-----------|:--:|:----:|:----:|:----:|
|✅  load    |    | reg  |     imm     |
|✅  push    |    | reg  |     N/A     |
|✅  pop     |    | reg  |     N/A     |
|🟥  aloc    |    | reg  |     N/A     |
|✅  add     |    | reg  | reg  | reg  |
|✅  sub     |    | reg  | reg  | reg  |
|✅  div     |    | reg  | reg  | reg  |
|✅  mul     |    | reg  | reg  | reg  |
|✅  eq      |    | reg  | reg  | N/A  |
|🟥  neq     |    | reg  | reg  | N/A  |
|🟥  gt      |    | reg  | reg  | N/A  |
|🟥  geq     |    | reg  | reg  | N/A  |
|🟥  lt      |    | reg  | reg  | N/A  |
|🟥  leq     |    | reg  | reg  | N/A  |
|🟥  setm    |    | reg  | reg  | N/A  |
|✅  inc     |    | reg  |     N/A     |
|🟥  dec     |    | reg  |     N/A     |
|✅  printreg|    | reg  |     N/A     |
|🟥  jmp     |    |      label name    |
|🟥  jeq     |    |      label name    |
|✅  jne     |    |      label name    |
|✅  hult    |    |        N/A         |
|🟥  nop     |    |        N/A         |
|🟥  ige     |    |        N/A         |
|🟥  not     |    | reg  |     N/A     |
|✅  and     |    | reg  | reg  | reg  |
|✅  or      |    | reg  | reg  | reg  |
|🟥  xor     |    | reg  | reg  | reg  |
|🟥  shl     |    | reg  | reg  | reg  |
|🟥  shr     |    | reg  | reg  | reg  |
|🟥  sar     |    | reg  | reg  | reg  |
|🟥  rol     |    | reg  | reg  | reg  |
|🟥  ror     |    | reg  | reg  | reg  |
