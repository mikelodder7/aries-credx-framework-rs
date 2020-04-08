use super::{AttributeEncoder, BITS_IN_ZERO};

use amcl_wrapper::field_elem::FieldElement;

impl AttributeEncoder for FieldElement {
    type Output = FieldElement;

    fn max() -> Self::Output {
        let co: amcl_wrapper::types::BigNum = *amcl_wrapper::constants::CurveOrder;
        FieldElement::from(co)
    }

    fn zero_center() -> Self::Output {
        FieldElement::one().shift_left(BITS_IN_ZERO)
    }

    fn from_vec(bytes: Vec<u8>) -> Self::Output {
        let mut data;
        if bytes.len() < amcl_wrapper::constants::FieldElement_SIZE {
            data = vec![0u8; amcl_wrapper::constants::FieldElement_SIZE - bytes.len()];
            data.extend_from_slice(&bytes); 
        } else if bytes.len() > amcl_wrapper::constants::FieldElement_SIZE {
            data = vec![0u8; amcl_wrapper::constants::FieldElement_SIZE];
            data.copy_from_slice(&bytes[..amcl_wrapper::constants::FieldElement_SIZE]);
        } else {
            data = vec![0u8; amcl_wrapper::constants::FieldElement_SIZE];
            data.copy_from_slice(bytes.as_slice());
        }
        FieldElement::from_bytes(data.as_slice()).map_err(|e| format!("{:?}", e)).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rfc3339_string_convert() {
        let res = FieldElement::encode_from_rfc3339_as_unixtimestamp("2018-01-26T18:30:09.453+00:00");
        assert!(res.is_ok());
        assert_eq!(FieldElement::from(1_516_991_409u64) + FieldElement::zero_center(), res.unwrap());

        let res = FieldElement::encode_from_rfc3339_as_unixtimestamp("2020-01-26T00:30:09.000+18:00");
        assert!(res.is_ok());
        assert_eq!(FieldElement::from(1_579_933_809u64) + FieldElement::zero_center(), res.unwrap());

        let res = FieldElement::encode_from_rfc3339_as_unixtimestamp("1970-01-01T00:00:00.000+00:00");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FieldElement::zero_center());

        let res = FieldElement::encode_from_rfc3339_as_unixtimestamp("1900");
        assert!(res.is_err());

        let res = FieldElement::encode_from_rfc3339_as_dayssince1900("1982-12-20T10:45:00.000-06:00");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FieldElement::zero_center() + FieldElement::from(30303));
    }

    #[test]
    fn decimal_test() {
        let res1 = FieldElement::encode_from_f64(1.33f32);
        assert!(res1.is_ok());
        let res2 = FieldElement::encode_from_f64(-1.33f32);
        assert!(res2.is_ok());
        assert_eq!(FieldElement::zero_center(), res1.unwrap() + res2.unwrap());

        let res1 = FieldElement::encode_from_f64(std::f64::MAX);
        assert!(res1.is_ok());
        let res2 = res1.unwrap();
        assert_eq!((&res2 - &res2), FieldElement::new());

        let res3 = FieldElement::encode_from_f64(std::f64::MIN);
        assert!(res3.is_ok());
        assert_eq!(FieldElement::zero_center(), &res3.unwrap() + &res2);

        let res1 = FieldElement::encode_from_f64(std::f64::NEG_INFINITY);
        assert!(res1.is_ok());
        assert_eq!(FieldElement::from(8), res1.unwrap());

        let pos_inf = <FieldElement as AttributeEncoder>::max() - FieldElement::from(9u64);
        let res1 = FieldElement::encode_from_f64(std::f64::INFINITY);
        assert!(res1.is_ok());
        assert_eq!(pos_inf, res1.unwrap());

        let res1 = FieldElement::encode_from_f64(std::f64::NAN);
        assert!(res1.is_ok());
        let res2 = <FieldElement as AttributeEncoder>::max() - FieldElement::from(8u64);
        assert_eq!(res2, res1.unwrap());
    }

    #[test]
    fn size_test() {
        let mut test_vectors = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_vectors.push("test_vectors");
        test_vectors.push("integers.txt");
        let lines = std::fs::read_to_string(test_vectors).unwrap().split("\n").map(|s| s.to_string()).collect::<Vec<String>>();
        assert_eq!(lines.len(), 7);
        for i in 0..lines.len() - 2 {
            let parts = lines[i].split(",").collect::<Vec<&str>>();
            let value = parts[0].parse::<isize>().unwrap();
            let expected = FieldElement::from_hex(parts[1].to_string()).unwrap();
            let res = FieldElement::encode_from_isize(value);
            assert!(res.is_ok());
            assert_eq!(expected, res.unwrap());
        }
        let parts = lines[lines.len() - 2].split(",").collect::<Vec<&str>>();
        let value = parts[0].parse::<usize>().unwrap();
        let expected = FieldElement::from_hex(parts[1].to_string()).unwrap();
        let res = FieldElement::encode_from_usize(value);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());

        let parts = lines[lines.len() - 1].split(",").collect::<Vec<&str>>();
        assert_eq!(parts[0], "null");
        assert_eq!(FieldElement::from_hex(parts[1].to_string()).unwrap(), FieldElement::encoded_null().unwrap());
    }
}