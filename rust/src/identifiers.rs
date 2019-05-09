use crate::errors::*;
use data_encoding::BASE32_NOPAD;
use regex::Regex;
use std::str::FromStr;
use std::{convert, fmt};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FatcatId(Uuid);

impl fmt::Display for FatcatId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", uuid2fcid(&self.to_uuid()))
    }
}

impl FromStr for FatcatId {
    type Err = Error;
    fn from_str(s: &str) -> Result<FatcatId> {
        fcid2uuid(s).map(FatcatId)
    }
}

impl convert::AsRef<Uuid> for FatcatId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl convert::Into<Uuid> for FatcatId {
    fn into(self) -> Uuid {
        self.0
    }
}

impl convert::From<Uuid> for FatcatId {
    fn from(u: Uuid) -> FatcatId {
        FatcatId(u)
    }
}

impl FatcatId {
    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
    // TODO: make it possible to just pass 'Uuid' in addition to '&Uuid'
    pub fn from_uuid(u: &Uuid) -> FatcatId {
        FatcatId(*u)
    }
}

/// Convert fatcat IDs (base32 strings) to UUID
pub fn fcid2uuid(fcid: &str) -> Result<Uuid> {
    if fcid.len() != 26 {
        return Err(FatcatError::InvalidFatcatId(fcid.to_string()).into());
    }
    let mut raw = vec![0; 16];
    BASE32_NOPAD
        .decode_mut(fcid.to_uppercase().as_bytes(), &mut raw)
        .map_err(|_dp| FatcatError::InvalidFatcatId(fcid.to_string()))?;
    // unwrap() is safe here, because we know raw is always 16 bytes
    Ok(Uuid::from_bytes(&raw).unwrap())
}

/// Convert UUID to fatcat ID string (base32 encoded)
pub fn uuid2fcid(id: &Uuid) -> String {
    let raw = id.as_bytes();
    BASE32_NOPAD.encode(raw).to_lowercase()
}

pub fn check_username(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[A-Za-z][A-Za-z0-9._-]{2,24}$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "username (expected, eg, 'AcidBurn')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_username() {
    assert!(check_username("bnewbold").is_ok());
    assert!(check_username("BNEWBOLD").is_ok());
    assert!(check_username("admin").is_ok());
    assert!(check_username("friend-bot").is_ok());
    assert!(check_username("dog").is_ok());
    assert!(check_username("g_____").is_ok());
    assert!(check_username("bnewbold2-archive").is_ok());
    assert!(check_username("bnewbold2-internetarchive").is_ok());

    assert!(check_username("").is_err());
    assert!(check_username("_").is_err());
    assert!(check_username("gg").is_err());
    assert!(check_username("adminadminadminadminadminadminadmin").is_err());
    assert!(check_username("bryan newbold").is_err());
    assert!(check_username("01234567-3456-6780").is_err());
    assert!(check_username(".admin").is_err());
    assert!(check_username("-bot").is_err());
}

pub fn check_pmcid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^PMC\d+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "PubMed Central ID (PMCID) (expected, eg, 'PMC12345')".to_string(),
            raw.to_string(),
        ))?
    }
}

pub fn check_pmid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "PubMed ID (PMID) (expected, eg, '1234')".to_string(),
            raw.to_string(),
        ))?
    }
}

pub fn check_wikidata_qid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^Q\d+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "Wikidata QID (expected, eg, 'Q1234')".to_string(),
            raw.to_string(),
        ))?
    }
}

pub fn check_doi(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^10.\d{3,6}/.+$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "DOI (expected, eg, '10.1234/aksjdfh')".to_string(),
            raw.to_string(),
        ))?
    }
}

pub fn check_issn(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d{4}-\d{3}[0-9X]$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "ISSN (expected, eg, '1234-5678')".to_string(),
            raw.to_string(),
        ))?
    }
}

pub fn check_orcid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d{4}-\d{4}-\d{4}-\d{3}[\dX]$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "ORCID (expected, eg, '0123-4567-3456-6789')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_orcid() {
    assert!(check_orcid("0123-4567-3456-6789").is_ok());
    assert!(check_orcid("0123-4567-3456-678X").is_ok());
    assert!(check_orcid("01234567-3456-6780").is_err());
    assert!(check_orcid("0x23-4567-3456-6780").is_err());
}

pub fn check_md5(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-f0-9]{32}$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedChecksum(
            "MD5".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_md5() {
    assert!(check_md5("1b39813549077b2347c0f370c3864b40").is_ok());
    assert!(check_md5("1g39813549077b2347c0f370c3864b40").is_err());
    assert!(check_md5("1B39813549077B2347C0F370c3864b40").is_err());
    assert!(check_md5("1b39813549077b2347c0f370c3864b4").is_err());
    assert!(check_md5("1b39813549077b2347c0f370c3864b411").is_err());
}

pub fn check_sha1(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-f0-9]{40}$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedChecksum(
            "SHA-1".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_sha1() {
    assert!(check_sha1("e9dd75237c94b209dc3ccd52722de6931a310ba3").is_ok());
    assert!(check_sha1("g9dd75237c94b209dc3ccd52722de6931a310ba3").is_err());
    assert!(check_sha1("e9DD75237C94B209DC3CCD52722de6931a310ba3").is_err());
    assert!(check_sha1("e9dd75237c94b209dc3ccd52722de6931a310ba").is_err());
    assert!(check_sha1("e9dd75237c94b209dc3ccd52722de6931a310ba33").is_err());
}

pub fn check_sha256(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-f0-9]{64}$").unwrap();
    }
    if RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedChecksum(
            "SHA-256".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_sha256() {
    assert!(
        check_sha256("cb1c378f464d5935ddaa8de28446d82638396c61f042295d7fb85e3cccc9e452").is_ok()
    );
    assert!(
        check_sha256("gb1c378f464d5935ddaa8de28446d82638396c61f042295d7fb85e3cccc9e452").is_err()
    );
    assert!(
        check_sha256("UB1C378F464d5935ddaa8de28446d82638396c61f042295d7fb85e3cccc9e452").is_err()
    );
    assert!(
        check_sha256("cb1c378f464d5935ddaa8de28446d82638396c61f042295d7fb85e3cccc9e45").is_err()
    );
    assert!(
        check_sha256("cb1c378f464d5935ddaa8de28446d82638396c61f042295d7fb85e3cccc9e4522").is_err()
    );
}

pub fn check_release_type(raw: &str) -> Result<()> {
    let valid_types = vec![
        // Citation Style Language official types
        "article",
        "article-magazine",
        "article-newspaper",
        "article-journal",
        "bill",
        "book",
        "broadcast",
        "chapter",
        "dataset",
        "entry",
        "entry-dictionary",
        "entry-encyclopedia",
        "figure",
        "graphic",
        "interview",
        "legislation",
        "legal_case",
        "manuscript",
        "map",
        "motion_picture",
        "musical_score",
        "pamphlet",
        "paper-conference",
        "patent",
        "post",
        "post-weblog",
        "personal_communication",
        "report",
        "review",
        "review-book",
        "song",
        "speech",
        "thesis",
        "treaty",
        "webpage",
        // fatcat-specific extensions
        "peer_review",
        "software",
        "standard",
        "abstract",
        "editorial",
        "letter",
        "stub",
    ];
    for good in valid_types {
        if raw == good {
            return Ok(());
        }
    }
    Err(FatcatError::NotInControlledVocabulary(
        "release_type (should be valid CSL, like 'article-journal')".to_string(),
        raw.to_string(),
    ))?
}

#[test]
fn test_check_release_type() {
    assert!(check_release_type("book").is_ok());
    assert!(check_release_type("article-journal").is_ok());
    assert!(check_release_type("standard").is_ok());
    assert!(check_release_type("journal-article").is_err());
    assert!(check_release_type("BOOK").is_err());
    assert!(check_release_type("book ").is_err());
}

pub fn check_release_stage(raw: &str) -> Result<()> {
    let valid_types = vec![
        // DRIVER types (minus "version" suffix)
        "draft",
        "submitted",
        "accepted",
        "published",
        "updated",
        // fatcat-specific extensions
        "retraction",
    ];
    for good in valid_types {
        if raw == good {
            return Ok(());
        }
    }
    Err(FatcatError::NotInControlledVocabulary(
        "release_stage".to_string(),
        raw.to_string(),
    ))?
}

#[test]
fn test_check_release_stage() {
    assert!(check_release_stage("draft").is_ok());
    assert!(check_release_stage("retraction").is_ok());
    assert!(check_release_stage("published").is_ok());
    assert!(check_release_stage("pre-print").is_err());
    assert!(check_release_stage("DRAFT").is_err());
    assert!(check_release_stage("draft ").is_err());
}

pub fn check_contrib_role(raw: &str) -> Result<()> {
    let valid_types = vec![
        // Citation Style Language official role types
        "author",
        "collection-editor",
        "composer",
        "container-author",
        "director",
        "editor",
        "editorial-director",
        "editortranslator",
        "illustrator",
        "interviewer",
        "original-author",
        "recipient",
        "reviewed-author",
        "translator",
        // common extension (for conference proceeding chair)
        //"chair",
    ];
    for good in valid_types {
        if raw == good {
            return Ok(());
        }
    }
    Err(FatcatError::NotInControlledVocabulary(
        "contrib.role (should be valid CSL, like 'author', 'editor')".to_string(),
        raw.to_string(),
    ))?
}

#[test]
fn test_check_contrib_role() {
    assert!(check_contrib_role("author").is_ok());
    assert!(check_contrib_role("editor").is_ok());
    assert!(check_contrib_role("chair").is_err());
    assert!(check_contrib_role("EDITOR").is_err());
    assert!(check_contrib_role("editor ").is_err());
}

// TODO: make the above checks "more correct"
// TODO: check ISBN-13
