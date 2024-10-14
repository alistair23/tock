/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

MEMORY
{
  rom (rx)  : ORIGIN = 0x00026000, LENGTH = 360K
  prog (rx) : ORIGIN = 0x00080000, LENGTH = 792K - 360K
  ram (rwx) : ORIGIN = 0x20006000, LENGTH = 232K
}

PAGE_SIZE = 4K;

INCLUDE tock_kernel_layout.ld
