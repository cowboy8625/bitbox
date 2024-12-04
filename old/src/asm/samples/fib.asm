.entry main
main:
    load[u32] %0  1 ; a = 1
    load[u32] %1  1 ; b = 1
    load[u32] %2 46 ; c = 46 (the number of iterations)
    load[u32] %3  2 ; d = 2 (to start counting from the third Fibonacci number)
loop:
    push[u32] %1    ; push b to stack
    add[u32] %1 %0 %1 ; b = a + b
    pop[u32] %0     ; a = old b (from stack)
    inc[u32] %3     ; d++
    jne %3 %2 loop  ; if d != c, jump to loop
    printreg[u32] %1 ; print b (the last computed Fibonacci number)
    hult
