# ezcheck

ezcheck(easy check) is an ergonomic, standard-output command-line tool for calculating, comparing, and verifying the hash of strings and files.

ezcheck have two backends: [ring](https://docs.rs/ring) and [hashes](https://docs.rs/hashes), and you can only choose one of them. The main differences between them are:

| Features             | ring                                                       | hashes                                                          |
|----------------------|------------------------------------------------------------|-----------------------------------------------------------------|
| Speed                | Fast                                                       | About 10 times slower than ring.                                |
| Supported algorithms | SHA256, SHA384, SHA512, SHA512/256                         | MD2, MD4, MD5, SHA1, SHA224, SHA256, SHA384, SHA512, SHA512/256 |
| Implement languages  | Assembly, Rust, C and etc..                                | Rust                                                            |
| Compatibility        | May not work on every machine with different architecture. | Works well with Rust.                                           |

⚠️ Please notice that although ezcheck(hashes backend) supports a lot of hash algorithms, `MD2`, `MD4`, `MD5`, `SHA1` are proven to be **insecure**. ezcheck still provides them for maximum compatibility, but **it does not recommend users continue to use them**. 

## Build

### Requirements

* [Rust 1.71.0+](https://www.rust-lang.org/)

### Build

```bash
$ git clone https://github.com/Metaphorme/ezcheck
$ cd ezcheck
$ # Choose one from ring backend or hashes backend
$ # ring backend
$ cargo build --release --features ring_backend
$ # hashes backend
$ cargo build --release --features hashes_backend
$
$ ./target/release/ezcheck --version
```

### Run tests

```bash
$ cargo test --features ring_backend    # ring backend
$ cargo test --features hashes_backend  # hashes backend
```

## Usage

Supported hash algorithms of different backends:

| ring       | hashes     |
|------------|------------|
|            | MD2        |
|            | MD4        |
|            | MD5        |
|            | SHA1       |
|            | SHA224     |
| SHA256     | SHA256     |
| SHA384     | SHA384     |
| SHA512     | SHA512     |
| SHA512     | SHA512     |
| SHA512/256 | SHA512/256 |

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
$ ezcheck compare -f image.jpg -c bebc102992450c68e5543383889e27c9
INFO: Hash Algorithm could be MD5, MD4, MD2
MD5 FAILED  Current Hash:  cb74bb502cc0949aad5cd838f91f0623
MD4 OK
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
$ cat sha256sum.txt
9ec44ac67ab1e1c98fe0406478d5297d  滕王阁序.txt
bebc102992450c68e5543383889e27c9  image.jpg
$ ezcheck check -c sha256sum.txt 
滕王阁序.txt: MD5 FAILED  Current Hash:  07c4e6a2c2db5f2d3a8998a3dba84a96
滕王阁序.txt: MD4 OK
image.jpg: MD5 FAILED  Current Hash:  cb74bb502cc0949aad5cd838f91f0623
image.jpg: MD4 OK

# Actually, ezcheck supports various algorithm in the same check file in auto detect.
# 🤔 But why this happens?
$ cat sha256sum.txt
00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95  滕王阁序.txt
4c03795a6bca220a68eae7c4f136d6247d58671e074bccd58a3b9989da55f56f *image.jpg
9ec44ac67ab1e1c98fe0406478d5297d  滕王阁序.txt

$ ezcheck check -c sha256sum.txt
滕王阁序.txt: SHA256 OK
image.jpg: SHA256 OK
滕王阁序.txt: MD5 FAILED  Current Hash:  07c4e6a2c2db5f2d3a8998a3dba84a96
滕王阁序.txt: MD4 OK
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
