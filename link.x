ENTRY(_start)
MEMORY {
  program (rwx) : ORIGIN = 0x80200000, LENGTH = 2 * 1024 * 1024
}

SECTIONS {
  .text : {
    *(.text)
  } > program

  .data : {
    *(.data)
  } > program

  .rodata : {
    *(.rodata)
  } > program

  .bss : {
    *(.bss)
  } > program
}