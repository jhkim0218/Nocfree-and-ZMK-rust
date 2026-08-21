MEMORY
{
  /* Preserved MBR + S140 end at 0x27000. */
  FLASH : ORIGIN = 0x00027000, LENGTH = 0x0003E000

  /* S140 v7 reserves the lower 64 KiB of RAM on this two-link configuration. */
  RAM : ORIGIN = 0x20010000, LENGTH = 0x00010000
}
