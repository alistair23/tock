# Licensed under the Apache License, Version 2.0 or the MIT License.
# SPDX-License-Identifier: Apache-2.0 OR MIT
# Copyright Tock Contributors 2022.

# Makefile for building the tock kernel for the CLUE nRF52840 board.

include ../Makefile.common

ifdef PORT
  FLAGS += -p $(PORT)
endif

# Upload the kernel using nrfutil
.PHONY: program program-apps

# Default target for installing the kernel.
.PHONY: install
install: program

program: $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin
ifeq ($(PORT),) 
	$(error Please specify the serial port using the PORT variable)
endif
	adafruit-nrfutil dfu genpkg --application $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).bin $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).zip
	adafruit-nrfutil --verbose dfu serial -pkg $(TOCK_ROOT_DIRECTORY)target/$(TARGET)/release/$(PLATFORM).zip $(FLAGS) -b 115200 --singlebank --touch 1200
