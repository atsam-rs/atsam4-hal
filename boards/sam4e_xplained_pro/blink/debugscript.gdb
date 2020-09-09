target remote :2331

# (JLink - enable semihosting)
monitor semihosting enable
# (Jlink - output to GDB)  
monitor semihosting IOClient 2

load
break main
continue
