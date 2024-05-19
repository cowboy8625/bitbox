.entry main

main:
  load[u32] %0 123
  load[u32] %1 321
  add[u32] %0 %0 %1
  hult
