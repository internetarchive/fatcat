extern crate fatcat;
extern crate uuid;

// TODO: these should just be in-line in identifiers.rs

use fatcat::identifiers::{fcid2uuid, uuid2fcid, FatcatId};
use uuid::Uuid;

#[test]
fn test_fcid_conversions() {
    let test_uuid = Uuid::parse_str("86daea5b-1b6b-432a-bb67-ea97795f80fe").unwrap();
    let test_str = "q3nouwy3nnbsvo3h5klxsx4a7y";

    assert_eq!(test_str, uuid2fcid(&test_uuid));
    assert_eq!(test_uuid, fcid2uuid(test_str).unwrap());
    assert_eq!(test_uuid, fcid2uuid(&test_str.to_uppercase()).unwrap());
    assert_eq!(test_uuid, fcid2uuid(&uuid2fcid(&test_uuid)).unwrap());

    assert_eq!(
        Uuid::parse_str("10842108-4210-8421-0842-108421084210").unwrap(),
        fcid2uuid("ccccccccccccccccccccccccca").unwrap()
    );

    assert_eq!(false, fcid2uuid("asdf").is_ok());
    assert_eq!(false, fcid2uuid("q3nouwy3nnbsvo3h5klx").is_ok());
    assert_eq!(false, fcid2uuid("10Oouwy3nnbsvo3h5klxsx4a7y").is_ok());
    assert_eq!(false, fcid2uuid("cccccccccccccccccccccccccc").is_ok());
}

#[test]
fn test_fcid_struct() {
    let test_uuid = Uuid::parse_str("86daea5b-1b6b-432a-bb67-ea97795f80fe").unwrap();
    let test_str = "q3nouwy3nnbsvo3h5klxsx4a7y";
    let test_fcid = FatcatId::from_uuid(&test_uuid);

    assert_eq!(test_str, test_fcid.to_string());
    assert_eq!(test_str, format!("{}", test_fcid));
    assert_eq!(test_uuid, test_fcid.to_uuid());

    // Inner UUID isn't public, so this doesn't work
    //let test_fcid2 = FatcatId(test_uuid);
    //assert_eq!(test_fcid, test_fcid2);
}
