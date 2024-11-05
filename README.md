# ezcheck

ezcheck is an ergonomic, standard-output command-line tool for calculating, comparing, and verifying the hashes of strings and files.

ezcheck supports a lot of hash algorithm: MD2(Unsafe), MD4(Unsafe), MD5(Unsafe), SHA1(Unsafe), SHA224, SHA256, SHA384, SHA512. Although many of the hash algorithms are proven to be insecure, ezcheck still provides them for maximum compatibility, but it does not recommend users continue to use them. 

## Build

### Requirements

* [Rust](https://www.rust-lang.org/)

### Build

```bash
$ git clone https://github.com/Metaphorme/ezcheck
$ cd ezcheck
$ cargo build --release
$ ./target/release/ezcheck --version
```

### Run tests

```bash
$ cargo test
```

## Usage

Supported hash algorithm:
* MD2(Unsafe)
* MD4(Unsafe)
* MD5(Unsafe)
* SHA1(Unsafe)
* SHA224
* SHA256
* SHA384
* SHA512

### Calculate

Calculate hash for a file or text.

```bash
Usage:
    ezcheck calculate [ALGORITHM (Default SHA256)] (-f file | -t text)

Examples:
$ ezcheck calculate sha256 -f image.jpg
4c03795a6bca220a68eae7c4f136d6247d58671e074bccd58a3b9989da55f56f  image.jpg

$ ezcheck calculate sha256 -t "Hello"
SHA256:  185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969

$ ezcheck calculate -f image.jpg
No algorithm specified. Using SHA256 as the default.
4c03795a6bca220a68eae7c4f136d6247d58671e074bccd58a3b9989da55f56f  image.jpg

# We could also redirect the output into a file, just like shasum does.
$ ezcheck calculate sha256 -f image.jpg > sha256sum.txt
```

### Compare

Compare with given hash.

```bash
Usage:
  ezcheck compare [ALGORITHM (Leave blank to automatically detect algorithm)] (-f file | -t text) -c hash
  
Examples:
$ ezcheck compare sha256 -f image.jpg -c 4c03795a6bca220a68eae7c4f136d6247d58671e074bccd58a3b9989da55f56f
SHA256 OK

$ ezcheck compare sha256 -t "Hello" -c 085f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969
SHA256 FAILED  Current Hash:  185f8db32271fe25f561a6fc938b2e264306ec304eda518007d1764826381969

# Auto detect hash algorithm
$ ezcheck compare -f image.jpg -c cb74bb502cc0949aad5cd838f91f0623 
INFO: Hash Algorithm could be MD2, MD4, MD5
MD2 FAILED  Current Hash:  10329710371ab70392948fcef544f728
MD4 FAILED  Current Hash:  bebc102992450c68e5543383889e27c9
MD5 OK
```

### Check

Check with given shasum file.

shasum file could be generated from [shasum](https://linux.die.net/man/1/shasum) and ezcheck. It looks like:

```
00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95  滕王阁序.txt
4c03795a6bca220a68eae7c4f136d6247d58671e074bccd58a3b9989da55f56f *image.jpg
```

```bash
Usage:
  ezcheck check [ALGORITHM (Leave blank to automatically detect algorithm)] -c check-file

Warning: The shasum file (or check file) should be in the same directory with files to be checked.
  
Example:
$ ezcheck check sha256 -c sha256sum.txt 
滕王阁序.txt: SHA256 OK
image.jpg: SHA256 OK

# Auto detect hash algorithm
$ ezcheck check -c sha256sum.txt 
滕王阁序.txt: SHA256 OK
image.jpg: SHA256 OK

# Actually, ezcheck supports various algorithm in the same check file in auto detect.
# 🤔 But why this happens?
$ cat sha256sum.txt
00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95  滕王阁序.txt
4c03795a6bca220a68eae7c4f136d6247d58671e074bccd58a3b9989da55f56f *image.jpg
cb74bb502cc0949aad5cd838f91f0623  image.jpg

$ ezcheck check -c sha256sum.txt
滕王阁序.txt: SHA256 OK
image.jpg: SHA256 OK
image.jpg: MD2 FAILED  Current Hash:  10329710371ab70392948fcef544f728
image.jpg: MD4 FAILED  Current Hash:  bebc102992450c68e5543383889e27c9
image.jpg: MD5 OK
```

## License

```
MIT License

Copyright (c) 2024 Heqi Liu

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
