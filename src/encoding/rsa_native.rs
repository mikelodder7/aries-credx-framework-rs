use super::{AttributeEncoder, BITS_IN_ZERO};

use openssl::{
    bn::{BigNum, BigNumRef}
};

use std::{cmp::{Eq, PartialEq}, ops::{Add, Sub, Neg}};

/// A simple wrapper class for converting attributes to cryptographic integers
/// represented in OpenSSL's BigNum library
#[derive(Debug)]
pub struct BigNumber(pub BigNum);

impl BigNumber {
    /// Convert a hex string to a BigNumber
    pub fn from_hex(n: &str) -> Result<BigNumber, String> {
        Ok(BigNumber(BigNum::from_hex_str(n).map_err(|_| format!("Unable to convert {} to a BigNumber", n))?))
    }
}

impl Add for BigNumber {
    type Output = Self;

    fn add(self, rhs: Self::Output) -> Self::Output {
        let mut bn = BigNum::new().unwrap();
        BigNumRef::checked_add(&mut bn, &self.0, &rhs.0).unwrap();
        BigNumber(bn)
    }
}

impl<'a, 'b> Add<&'b BigNumber> for &'a BigNumber {
    type Output = BigNumber;

    fn add(self, rhs: &'b BigNumber) -> BigNumber {
        let mut bn = BigNum::new().unwrap();
        BigNumRef::checked_add(&mut bn, &self.0, &rhs.0).unwrap();
        BigNumber(bn)
    }
}

impl Neg for BigNumber {
    type Output = BigNumber;

    fn neg(self) -> Self::Output {
        let mut bn = BigNum::from_slice(&self.0.to_vec()).unwrap();
        bn.set_negative(if self.0.is_negative() { false } else { true });
        BigNumber(bn)
    }
}

impl Sub for BigNumber {
    type Output = Self;

    fn sub(self, rhs: Self::Output) -> Self::Output {
        let mut bn = BigNum::new().unwrap();
        BigNumRef::checked_sub(&mut bn, &self.0, &rhs.0).unwrap();
        BigNumber(bn)
    }
}

impl<'a, 'b> Sub<&'b BigNumber> for &'a BigNumber {
    type Output = BigNumber;

    fn sub(self, rhs: &'b Self::Output) -> Self::Output {
        let mut bn = BigNum::new().unwrap();
        BigNumRef::checked_sub(&mut bn, &self.0, &rhs.0).unwrap();
        BigNumber(bn)
    }
}

impl From<u64> for BigNumber {
    fn from(v: u64) -> Self {
        BigNumber(BigNum::from_slice(&v.to_be_bytes()[..]).unwrap())
    }
}

impl From<i64> for BigNumber {
    fn from(v: i64) -> Self { BigNumber(BigNum::from_slice(&v.to_be_bytes()[..]).unwrap()) }
}

impl PartialEq for BigNumber {
    fn eq(&self, other: &BigNumber) -> bool {
        self.0 == other.0
    }
}

impl Eq for BigNumber{}

impl AttributeEncoder for BigNumber {
    type Output = BigNumber;

    fn max() -> Self::Output {
        let bytes = vec![0xFF; 32];
        Self(BigNum::from_slice(bytes.as_slice()).unwrap())
    }

    fn zero_center() -> Self::Output {
        let mut bn = BigNum::new().unwrap();
        bn.set_bit(BITS_IN_ZERO as i32).unwrap();
        Self(bn)
    }

    fn from_vec(bytes: Vec<u8>) -> Self::Output {
        Self(BigNum::from_slice(bytes.as_slice()).unwrap()) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rfc3339_string_convert() {
        let res = BigNumber::encode_from_rfc3339_as_unixtimestamp("2018-01-26T18:30:09.453+00:00");
        assert!(res.is_ok());
        assert_eq!(BigNumber::from(1_516_991_409u64) + BigNumber::zero_center(), res.unwrap());

        let res = BigNumber::encode_from_rfc3339_as_unixtimestamp("2020-01-26T00:30:09.000+18:00");
        assert!(res.is_ok());
        assert_eq!(BigNumber::from(1_579_933_809u64) + BigNumber::zero_center(), res.unwrap());

        let res = BigNumber::encode_from_rfc3339_as_unixtimestamp("1970-01-01T00:00:00.000+00:00");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), BigNumber::zero_center());

        let res = BigNumber::encode_from_rfc3339_as_unixtimestamp("1900");
        assert!(res.is_err());
    }

    #[test]
    fn decimal_test() {
        let res1 = BigNumber::encode_from_f64(1.33f32);
        assert!(res1.is_ok());
        let res2 = BigNumber::encode_from_f64(-1.33f32);
        assert!(res2.is_ok());
        assert_eq!(BigNumber::zero_center(), res1.unwrap() + res2.unwrap());

        let res1 = BigNumber::encode_from_f64(std::f64::MAX);
        assert!(res1.is_ok());
        let res2 = res1.unwrap();
        assert_eq!((&res2 - &res2).0, BigNum::new().unwrap());

        let res3 = BigNumber::encode_from_f64(std::f64::MIN);
        assert!(res3.is_ok());
        assert_eq!(BigNumber::zero_center(), &res3.unwrap() + &res2);

        let res1 = BigNumber::encode_from_f64(std::f64::NEG_INFINITY);
        assert!(res1.is_ok());
        assert_eq!(BigNum::from_u32(8).unwrap(), res1.unwrap().0);

        let pos_inf = BigNumber::max() - BigNumber::from(9u64);
        let res1 = BigNumber::encode_from_f64(std::f64::INFINITY);
        assert!(res1.is_ok());
        assert_eq!(pos_inf, res1.unwrap());

        let res1 = BigNumber::encode_from_f64(std::f64::NAN);
        let nan = BigNumber::max() - BigNumber::from(8u64);
        assert!(res1.is_ok());
        assert_eq!(nan.0, res1.unwrap().0);
    }

    #[test]
    fn size_test() {
        let mut test_vectors = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_vectors.push("test_vectors");
        test_vectors.push("integers.txt");
        let lines = std::fs::read_to_string(test_vectors).unwrap().split("\n").map(|s| s.to_string()).collect::<Vec<String>>();
        assert_eq!(lines.len(), 7);
        for i in 0..lines.len() - 1 {
            let parts = lines[i].split(",").collect::<Vec<&str>>();
            let value = parts[0].parse::<isize>().unwrap();
            let expected = BigNumber::from_hex(parts[1]).unwrap();
            let res = BigNumber::encode_from_isize(value);
            assert!(res.is_ok());
            assert_eq!(expected, res.unwrap());
        }
        let parts = lines[lines.len() - 1].split(",").collect::<Vec<&str>>();
        let value = parts[0].parse::<usize>().unwrap();
        let expected = BigNumber::from_hex(parts[1]).unwrap();
        let res = BigNumber::encode_from_usize(value);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());

        let parts = lines[lines.len() - 1].split(",").collect::<Vec<&str>>();
        assert_eq!(parts[0], "null");
        assert_eq!(BigNumber::from_hex(parts[1]).unwrap(), BigNumber::encoded_null().unwrap());
    }
}