#!/usr/bin/env python

from deadpool_dca import *

filters = [
    # Filter('data_R', ['R'], lambda stack_range, addr, size, data: size <= 4, lambda addr, size, data: data & 0xFF, '<B'),
    Filter('data_W', ['W'], lambda stack_range, addr, size, data: size <= 4, lambda addr, size, data: data & 0xFF, '<B'),
    # Filter('addr_R', ['R'], lambda stack_range, addr, size, data: size <= 4, lambda addr, size, data: addr & 0xFF, '<B'),
    # Filter('addr_W', ['W'], lambda stack_range, addr, size, data: size <= 4, lambda addr, size, data: addr & 0xFF, '<B'),
    # Filter('data_RW', ['W','R'], lambda stack_range, addr, size, data: size <= 4, lambda addr, size, data: data & 0xFF, '<B'),
    # Filter('addr_RW', ['R','R'], lambda stack_range, addr, size, data: size <= 4, lambda addr, size, data: addr & 0xFF, '<B')
]

T=TracerGrind('./aes_simple',
              filters=filters,
              arch=ARCH.amd64,
              addr_range='0x109080-0x109785')

T.run(100)
bin2daredevil(keywords=filters,
              configs={'attack_sbox':    {'algorithm':'AES', 'position':'LUT/AES_AFTER_SBOX'}})