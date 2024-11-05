pub mod calculator {
    use std::fmt;
    use std::io::{BufReader, Error, Read};
    use digest::DynDigest;
    use crate::extra::extra::bytes_to_hex;

    // https://github.com/rust-lang/rust/issues/47133
    #[allow(dead_code)]
    pub const BUFFER_SIZE: usize = 4096;

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub enum SupportedAlgorithm {
        MD2,
        MD4,
        MD5,
        SHA1,
        SHA224,
        SHA256,
        SHA384,
        SHA512,
    }

    impl fmt::Display for SupportedAlgorithm {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let algorithm = match self {
                SupportedAlgorithm::MD2 => "MD2",
                SupportedAlgorithm::MD4 => "MD4",
                SupportedAlgorithm::MD5 => "MD5",
                SupportedAlgorithm::SHA1 => "SHA1",
                SupportedAlgorithm::SHA224 => "SHA224",
                SupportedAlgorithm::SHA256 => "SHA256",
                SupportedAlgorithm::SHA384 => "SHA384",
                SupportedAlgorithm::SHA512 => "SHA512",
            };
            write!(f, "{}", algorithm)
        }
    }

    pub fn hash_calculator<R: Read>(
        mut reader: BufReader<R>,
        algorithm: SupportedAlgorithm)
    -> Result<String, Error> {

        let mut hasher: Box<dyn DynDigest> = match algorithm {
            SupportedAlgorithm::MD2 => Box::new(md2::Md2::default()),
            SupportedAlgorithm::MD4 => Box::new(md4::Md4::default()),
            SupportedAlgorithm::MD5 => Box::new(md5::Md5::default()),
            SupportedAlgorithm::SHA1 => Box::new(sha1::Sha1::default()),
            SupportedAlgorithm::SHA224 => Box::new(sha2::Sha224::default()),
            SupportedAlgorithm::SHA256 => Box::new(sha2::Sha256::default()),
            SupportedAlgorithm::SHA384 => Box::new(sha2::Sha384::default()),
            SupportedAlgorithm::SHA512 => Box::new(sha2::Sha512::default()),
        };

        let mut buffer = [0u8; BUFFER_SIZE];
        loop {
            match reader.read(&mut buffer) {
                Ok(read_bytes) => {
                    if read_bytes == 0 {
                        break;  // Finish reading the file
                    }
                    hasher.update(&buffer[..read_bytes]);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(bytes_to_hex(&*hasher.finalize_reset()))
    }
}

#[cfg(test)]
mod test_calculator {
    use std::io::BufReader;
    use std::fs::File;
    use super::calculator;

    const TEST_WORD: &[u8; 16] = b"Veni, vidi, vici";

    #[test]
    fn test_algorithm() {
        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::MD2).unwrap(),
            "3354cef96052efb872e8c0391a5cfb34"
        );

        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::MD4).unwrap(),
            "5c79b96c023c5a269ad205d33bce0f60"
        );

        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::MD5).unwrap(),
            "af1e16b12fec10c5ad09fb6478005b6c"
        );

        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::SHA1).unwrap(),
            "5df99149d56d7f82a9751ac4c36ada25d07f5e49"
        );

        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::SHA224).unwrap(),
            "9111df25d5715bc4ab42d6777f48d1bd592f7f991fbbc356ae370167"
        );

        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::SHA256).unwrap(),
            "b1610284c94bbf9aa78333e57ddce234a5e845d61e09ce91a7e19fa24737f466"
        );

        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::SHA384).unwrap(),
            "aed14590fa99f83c701236d63c50085faa8e57c7196846411dc595c42751e5e17d6bc10b767541d76eecdda086c5d4fc"
        );

        let reader = BufReader::new(&TEST_WORD[..]);
        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::SHA512).unwrap(),
            "6cf905a2c09fa2d9090f2712e2ae6d0fc8188cc845a1dc9dff4b3bd33e9d4fa43991cbb7cc3cf5d5aa8e32098796eb01e3f03c25c6ea863226e617ad6e5abec2"
        );
    }

    #[test]
    fn test_read_file() {
        let test_file = "tests/滕王阁序.txt";
        let file = File::open(test_file).unwrap();
        let reader = BufReader::new(file);

        assert_eq!(
            calculator::hash_calculator(reader, calculator::SupportedAlgorithm::SHA256).unwrap(),
            "00691413c731ee37f551bfaca6a34b8443b3e85d7c0816a6fe90aa8fc8eaec95"
        );
    }
}