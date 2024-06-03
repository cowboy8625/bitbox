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
|✅  copy    | 2    |      | Reg,Reg|
|✅  aloc    | 3    |      | Reg,Reg|
|✅  push    | 4    |      | Reg|
|✅  pop     | 5    |      | Reg|
|✅  add     | 6    |      | Reg,Reg,Reg|
|✅  sub     | 7    |      | Reg,Reg,Reg|
|✅  div     | 8    |      | Reg,Reg,Reg|
|✅  mul     | 9    |      | Reg,Reg,Reg|
|✅  inc     | 10   |      | Reg|
|✅  eq      | 11   |      | Reg,Reg|
|✅  jne     | 12   |      | Reg,Reg,Label|
|✅  hult    | 13   |      ||
|✅  printreg| 14   |      | Reg|
|✅  call    | 15   |      | Label|
|✅  and     | 16   |      | Reg,Reg,Reg|
|✅  or      | 17   |      | Reg,Reg,Reg|
|✅  shr     | 18   |      | Reg,Reg,Reg|
|✅  return  | 19   |      ||
|✅  syscall | 20   |      ||
|🟥  jmp     | N/A  |      | Label|
|🟥  jeq     | N/A  |      | Label|
|🟥  nop     | N/A  |      ||
|🟥  ige     | N/A  |      ||
|🟥  not     | N/A  |      | Reg|
|🟥  xor     | N/A  |      | Reg,Reg,Reg|
|🟥  shl     | N/A  |      | Reg,Reg,Reg|
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

| name | reg 0 |      reg 1    |     reg 2     |      reg 3    | reg 4 |
|:-----|:-----:|:-------------:|:-------------:|:-------------:|:-----:|
|WRITE | 0     | ptr to string | string length | static string | N/A   |

## Command Line Arguments

The command line arguments are placed in the heap.
Each argument pointer will be place on the stack.
Pointers will have the string length on the upper half of the pointer.
Register 0 will be set to the number of arguments.
### Example
Print all arguments given to program
```asm
.entry main
main:
    copy[u64] %31 %0 ; args length
    load[u64] %30 0  ; count
    load[u64] %0  1  ; length of string
    aloc[u64] %29 %0
    load[u8] %0  0xA  ; char `\n`
    store[u8] %29 %0

loop:
    ; get the ptr to string and length
    pop[u64] %5

    ; get ptr
    load[u64] %1 0b1111_1111_1111_1111_1111_1111_1111_1111
    and[u64] %1 %1 %5

    ; get length
    load[u64] %6 32
    shr[u64] %2 %5 %6 ; shift right by 32 bits

    ; write first arg
    load[u8] %0 0
    ; %1 ptr
    ; %2 length
    load[u8] %3 0
    syscall

    copy[u64] %1 %29 ; ptr
    load[u64] %2 1   ; length
    syscall

    inc[u64] %30
    jne %31 %30 loop

    hult
```
