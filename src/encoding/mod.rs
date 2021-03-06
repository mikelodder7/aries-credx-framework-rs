use chrono::DateTime;
use digest::{Digest, generic_array::typenum::U32};
use std::ops::{Add, Sub, Neg};

/// How many bits are used to shift 1 to get to zero centering
const BITS_IN_ZERO: usize = 254;

/// Represents an abstract encoder used for converting types to cryptographic integers
/// Cryptographic integers are limited to 256 bits
pub trait AttributeEncoder {
    /// The type to represent the cryptographic integer
    type Output: Add<Output = Self::Output> + Sub<Output = Self::Output> + Neg<Output = Self::Output> + From<u64>;

    /// Return the highest value for `Output`
    fn max() -> Self::Output;
    /// Return what a value that represents zero
    fn zero_center() -> Self::Output;
    /// Takes a vector of bytes and returns `Self::Output`
    fn from_vec(v: Vec<u8>) -> Self::Output;

    /// Encoded value to represent NULL values.
    /// Should indicate a value was not available
    fn encoded_null() -> Result<Self::Output, String> {
        let mut null = vec![0u8; 32];
        null[31] = 7;
        Ok(Self::from_vec(null))
    }

    /// Takes an date string that is formatted according to RFC3339
    /// and converts it to a cryptographic integer. 
    /// `value`: Any type that can be converted into a string slice
    fn encode_from_rfc3339_as_unixtimestamp<'a, A: Into<&'a str>>(value: A) -> Result<Self::Output, String> {
        let dt = DateTime::parse_from_rfc3339(value.into()).map_err(|e| format!("{:?}", e))?;
        Ok(Self::zero_center() + Self::Output::from(dt.timestamp() as u64))
    }

    /// Takes an date string that is formatted according to RFC3339
    /// and converts it to a cryptographic integer. 
    /// `value`: Any type that can be converted into a string slice
    fn encode_from_rfc3339_as_dayssince1900<'a, A: Into<&'a str>>(value: A) -> Result<Self::Output, String> {
        let dt = DateTime::parse_from_rfc3339(value.into()).map_err(|e| format!("{:?}", e))?;
        let base = DateTime::parse_from_rfc3339("1900-01-01T00:00:00.000+00:00").map_err(|e| format!("{:?}", e))?;
        Ok(Self::zero_center() + Self::Output::from((dt - base).num_days() as u64))
    }

    /// Takes a UTF-8 encoded string and uses the Blake2 hash to convert
    /// to a cryptographic integer.
    /// `value`: Any type that can be converted into a string slice.
    /// The hash can be anything that emits a 32 byte output.
    /// 
    /// An example call is encode_from_utf8_as_hash::<sha2::Sha256>("first_name")
    fn encode_from_utf8_as_hash<'a, A: Into<&'a str>, D: Digest<OutputSize = U32> + Default>(value: A) -> Result<Self::Output, String> {
        let hash = D::digest(value.into().as_bytes());
        Ok(Self::from_vec(hash[..].to_vec()))
    }

    /// Takes a 64-bit floating point number and converts it into
    /// a cryptographic integer
    /// `value`: Any type that can be converted into a f64
    fn encode_from_f64<A: Into<f64>>(v: A) -> Result<Self::Output, String> {
        use std::num::FpCategory::*;
        use num_bigint::Sign::*;

        let value = v.into();

        Ok(
            match value.classify() {
                Nan | Subnormal => { Self::max() - Self::Output::from(8) }
                Zero => Self::zero_center(),
                Infinite => {
                    if value.is_sign_positive() {
                        Self::max() - Self::Output::from(9)
                    } else {
                        Self::Output::from(8)
                    }
                },
                Normal => {
                    let mut b = bigdecimal::BigDecimal::from(value);

                    for _ in 0..BITS_IN_ZERO {
                        b = b.double();
                    }
                    let (_, mut d) = b.clone().into_bigint_and_exponent();
                    // 15 decimal places means no decimals at all
                    // Anything higher should be shifted
                    while d > 15 {
                        b = b.half();
                        d -= 1;
                    }
                    let (bi, _) = b.into_bigint_and_exponent();
                    let (sign, bytes) = bi.to_bytes_be();
                    match sign {
                        NoSign => Self::zero_center(),
                        Plus => Self::from_vec(bytes),
                        Minus => -Self::from_vec(bytes) + Self::zero_center()
                    }
                }
            }
        )
    }

    /// Takes a signed number and converts it into
    /// a cryptographic integer
    /// `value`: Any type that can be converted into a isize
    fn encode_from_isize<A: Into<isize>>(value: A) -> Result<Self::Output, String> {
        let value = value.into();
        if value < 0 {
            if value == std::isize::MIN {
                Ok(Self::zero_center() - Self::from_vec(value.to_be_bytes().to_vec()))
            } else {
                Ok(Self::zero_center() - Self::Output::from((-value) as u64))
            }
        } else {
            Ok(Self::zero_center() + Self::Output::from(value as u64))
        }
    }

    /// Takes an unsigned number and converts it into
    /// a cryptographic integer
    /// `value`: Any type that can be converted into a usize
    fn encode_from_usize<A: Into<usize>>(value: A) -> Result<Self::Output, String> {
        let value = value.into() as u64;
        Ok(Self::zero_center() + Self::from_vec(value.to_be_bytes().to_vec()))
    }
}


/// Provides an encoder to BLS12-381 FieldElements
#[cfg(feature = "bls381")]
pub mod bls381_fieldelem;

/// Provides an encoder to openssl's BIGNUM
#[cfg(feature = "rsa-native")]
pub mod rsa_native;