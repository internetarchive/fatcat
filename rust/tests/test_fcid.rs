extern crate fatcat;
extern crate uuid;

use fatcat::api_helpers::{fcid2uuid, uuid2fcid};
use uuid::Uuid;

#[test]
fn test_fcid_conversions() {
    let test_uuid = Uuid::parse_str("86daea5b-1b6b-432a-bb67-ea97795f80fe").unwrap();
    let test_fcid = "q3nouwy3nnbsvo3h5klxsx4a7y";

    assert_eq!(test_fcid, uuid2fcid(&test_uuid));
    assert_eq!(test_uuid, fcid2uuid(test_fcid).unwrap());
    assert_eq!(test_uuid, fcid2uuid(&test_fcid.to_uppercase()).unwrap());
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
