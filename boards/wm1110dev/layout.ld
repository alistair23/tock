/* Licensed under the Apache License, Version 2.0 or the MIT License. */
/* SPDX-License-Identifier: Apache-2.0 OR MIT                         */
/* Copyright Tock Contributors 2023.                                  */

/* prog: leave 16KB at the end for nonvolatile storage. */

MEMORY
{
  rom (rx)  : ORIGIN = 0x00010000, LENGTH = 256K
  prog (rx) : ORIGIN = 0x00050000, LENGTH = 704K-16K
  ram (rwx) : ORIGIN = 0x20000000, LENGTH = 256K
}

PAGE_SIZE = 4K;

INCLUDE tock_kernel_layout.ld
