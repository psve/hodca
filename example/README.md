`hodca` is designed to be used with the [Tracer](https://github.com/SideChannelMarvels/Tracer) software (either `TraceGrind` or `TracerPIN`). We assume that `TraceGrind` and `texttrace` is installed.

The file `aes_simple.c` contains a straight forward AES implementation with no protection. To compile it, run e.g.

```
gcc -O2 aes_simple -o aes_simple
```

We now want to collect a number of traces for the resulting binary in order to recover the key. To minimise the size of the traces, it's a good idea to do a preliminary trace. We do this by calling

```
valgrind --tool=tracergrind --output=aes_simple.trace ./aes_simple 00000000000000000000000000000000
```

This trace contains a lot of superfluous information. To find the address range that contains only our AES code, call

```
texttrace aes_simple.trace aes_simple.txt
tail aes_simple.txt
```

In our case, the last line of the above output is

```
[L] Loaded /path/to/binary/aes_simple from 0x0000000000109080 to 0x0000000000109785
```

We can then use the above range to filter our call too tracergrind, i.e. 

```
valgrind --tool=tracergrind --output=aes_simple.trace --filter=0x109080-0x109785 ./aes_simple 00000000000000000000000000000000
```

If you wish to further narrow down the range, we recommend using `TraceGraph` to do visual inspection. Now, we can use the `trace_it.py` script to collect a large number of traces. This will collect 100 traces into a single file as well as the plaintext inputs used to generate the traces, in our case the files `data_W_100_8096.trace` and `data_W_100_8096.input`. The first number denotes the number of traces, while the second number denotes the length of each trace. Note that we chose to trace the write operations; this can be change in `trace_it.py`. We can now use `hodca` to recover the key. 

```
hodca --correlation equality --data_type bytes --guess sbox --length 8096 --order 1 --path data_W_100_8096 --traces=100 --window 1
```

Which e.g. shows us that the first key byte is `0x10`:

```
Attacking key byte 0...
	Attacking all bits... Done! (0.0047 seconds)

Finished attacking key byte 0 in 0.0048 seconds.
	10, score = 100.0000
	41, score = 6.0000
	f3, score = 6.0000
	75, score = 5.0000
	91, score = 5.0000
	b4, score = 5.0000
	bb, score = 5.0000
	e5, score = 5.0000
	01, score = 4.0000
	02, score = 4.0000

	Lowest score: 2.0000
	Highest score: 100.0000

```