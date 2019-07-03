set pagination off
set remotetimeout unlimited
target extended-remote :3333

monitor halt

# print demangled symbols
set print asm-demangle on

# set backtrace limit to not have infinite backtrace loops
set backtrace limit 32

# break if we hit a panic
break rust_begin_unwind

# try to stop at the user entry point
break main

monitor arm semihosting enable
load
monitor reset halt
continue
