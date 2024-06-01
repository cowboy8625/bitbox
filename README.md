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
|✅  aloc    | 2    |      | Reg|
|✅  push    | 3    |      | Reg|
|✅  pop     | 4    |      | Reg|
|✅  add     | 5    |      | Reg,Reg,Reg|
|✅  sub     | 6    |      | Reg,Reg,Reg|
|✅  div     | 7    |      | Reg,Reg,Reg|
|✅  mul     | 8    |      | Reg,Reg,Reg|
|✅  inc     | 9    |      | Reg|
|✅  eq      | 10   |      | Reg,Reg|
|✅  jne     | 11   |      | Label|
|✅  hult    | 12   |      ||
|✅  printreg| 13   |      | Reg|
|✅  call    | 14   |      | Label|
|✅  and     | 15   |      | Reg,Reg,Reg|
|✅  or      | 16   |      | Reg,Reg,Reg|
|✅  return  | 17   |      ||
|✅  syscall | 18   |      ||
|🟥  jmp     | N/A  |      | Label|
|🟥  jeq     | N/A  |      | Label|
|🟥  nop     | N/A  |      ||
|🟥  ige     | N/A  |      ||
|🟥  not     | N/A  |      | Reg|
|🟥  xor     | N/A  |      | Reg,Reg,Reg|
|🟥  shl     | N/A  |      | Reg,Reg,Reg|
|🟥  shr     | N/A  |      | Reg,Reg,Reg|
|🟥  sar     | N/A  |      | Reg,Reg,Reg|
|🟥  rol     | N/A  |      | Reg,Reg,Reg|
|🟥  ror     | N/A  |      | Reg,Reg,Reg|
|🟥  neq     | N/A  |      | Reg,Reg|
|🟥  gt      | N/A  |      | Reg,Reg|
|🟥  geq     | N/A  |      | Reg,Reg|
|🟥  lt      | N/A  |      | Reg,Reg|
|🟥  leq     | N/A  |      | Reg,Reg|
|🟥  dec     | N/A  |      | Reg|

## Syscall

Register 0 is used for the syscall number, register 1-4 are used for arguments.

| name | reg 0 |      reg 1    |     reg 2     | reg 3 | reg 4 |
|:-----|:-----:|:-------------:|:-------------:|:-----:|:-----:|
|WRITE | 0     | ptr to string | string length | N/A   | N/A   |
