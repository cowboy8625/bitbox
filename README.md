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

Instructions consist of a 3 main parts.

1. **Code**: this is one byte
2. **Type**: this is one byte
  - if `type` is 0b1000_0000 then type is signed integer
  - if `type` is 0x00 then there are no args
3. **Args**: this is one byte to as many bytes as needed
  - **Arg 1**: this is one byte
  - **Arg 2**: this is two bytes
  - **Arg 3**: this is three bytes
  - **Arg 1 with Immediate Value**: this is one to as many bytes as needed. The type will dictate the size.

**Note**: opcode numbers may not be correct as they can change as we add more instructions

|instruction |opcode| type |                 data         |
|:-----------|:----:|:----:|:----------------------------:|
|✅  load    | 0    |      | Reg,Imm|
|✅  store   | 1    |      | Reg,Reg|
|✅  push    | 2    |      | Reg|
|✅  pop     | 3    |      | Reg|
|✅  aloc    | 4    |      | Reg|
|✅  add     | 5    |      | Reg,Reg,Reg|
|✅  sub     | 6    |      | Reg,Reg,Reg|
|✅  div     | 7    |      | Reg,Reg,Reg|
|✅  mul     | 8    |      | Reg,Reg,Reg|
|✅  eq      | 9    |      | Reg,Reg|
|🟥  neq     | 10   |      | Reg,Reg|
|🟥  gt      | 11   |      | Reg,Reg|
|🟥  geq     | 12   |      | Reg,Reg|
|🟥  lt      | 13   |      | Reg,Reg|
|🟥  leq     | 14   |      | Reg,Reg|
|🟥  setm    | 15   |      | Reg,Reg|
|✅  inc     | 16   |      | Reg|
|🟥  dec     | 17   |      | Reg|
|✅  printreg| 18   |      | Reg|
|🟥  jmp     | 19   |      | Label|
|🟥  jeq     | 20   |      | Label|
|✅  jne     | 21   |      | Label|
|✅  hult    | 22   |      ||
|🟥  nop     | 23   |      ||
|🟥  ige     | 24   |      ||
|🟥  not     | 25   |      | Reg|
|✅  and     | 26   |      | Reg,Reg,Reg|
|✅  or      | 27   |      | Reg,Reg,Reg|
|🟥  xor     | 28   |      | Reg,Reg,Reg|
|🟥  shl     | 29   |      | Reg,Reg,Reg|
|🟥  shr     | 30   |      | Reg,Reg,Reg|
|🟥  sar     | 31   |      | Reg,Reg,Reg|
|🟥  rol     | 32   |      | Reg,Reg,Reg|
|🟥  ror     | 33   |      | Reg,Reg,Reg|
|✅  call    | 34   |      | Label|
|✅  return  | 35   |      ||
