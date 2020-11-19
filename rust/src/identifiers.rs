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
    if fcid.is_ascii() == false || fcid.len() != 26 {
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
    if raw.is_ascii() && RE.is_match(raw) {
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
    assert!(check_username("bnewbßasdf").is_err());
    assert!(check_username("adminadminadminadminadminadminadmin").is_err());
    assert!(check_username("bryan newbold").is_err());
    assert!(check_username("01234567-3456-6780").is_err());
    assert!(check_username(".admin").is_err());
    assert!(check_username("-bot").is_err());
}

pub fn check_pmcid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^PMC\d+(\.\d+)?$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "PubMed Central ID (PMCID) (expected, eg, 'PMC12345')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_pmcid() {
    assert!(check_pmcid("PMC12345").is_ok());
    assert!(check_pmcid("PMC12345.1").is_ok());
    assert!(check_pmcid("PMC12345.96").is_ok());
    assert!(check_pmcid("PMC12345 ").is_err());
    assert!(check_pmcid("PMC").is_err());
    assert!(check_pmcid("PMC.3").is_err());
    assert!(check_pmcid("PMC1123.").is_err());
}

pub fn check_pmid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "PubMed ID (PMID) (expected, eg, '1234')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_pmid() {
    assert!(check_pmid("1234").is_ok());
    assert!(check_pmid("1234 ").is_err());
    assert!(check_pmid("").is_err());
    assert!(check_pmid("1.234").is_err());
    assert!(check_pmid("-1234").is_err());
    assert!(check_pmid(" 1234").is_err());
}

pub fn check_mag_id(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "Microsoft Academic Graph (mag) (expected, eg, '1234')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_mag_id() {
    assert!(check_mag_id("1234").is_ok());
    assert!(check_mag_id("1234 ").is_err());
    assert!(check_mag_id("").is_err());
    assert!(check_mag_id("1.234").is_err());
    assert!(check_mag_id("-1234").is_err());
    assert!(check_mag_id(" 1234").is_err());
}

pub fn check_jstor_id(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "JSTOR (jstor_id) (expected, eg, '1234')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_jstor_id() {
    assert!(check_jstor_id("1234").is_ok());
    assert!(check_jstor_id("1234 ").is_err());
    assert!(check_jstor_id("").is_err());
    assert!(check_jstor_id("1.234").is_err());
    assert!(check_jstor_id("-1234").is_err());
    assert!(check_jstor_id(" 1234").is_err());
}

pub fn check_core_id(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "CORE.ac.uk (core_id) (expected, eg, '1234')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_core_id() {
    assert!(check_core_id("1234").is_ok());
    assert!(check_core_id("1234 ").is_err());
    assert!(check_core_id("").is_err());
    assert!(check_core_id("1.234").is_err());
    assert!(check_core_id("-1234").is_err());
    assert!(check_core_id(" 1234").is_err());
}

pub fn check_wikidata_qid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^Q\d+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "Wikidata QID (expected, eg, 'Q1234')".to_string(),
            raw.to_string(),
        ))?
    }
}
#[test]
fn test_check_wikidata_qid() {
    assert!(check_wikidata_qid("Q1234").is_ok());
    assert!(check_wikidata_qid("Q1234 ").is_err());
    assert!(check_wikidata_qid("Q").is_err());
    assert!(check_wikidata_qid("Q1-234").is_err());
    assert!(check_wikidata_qid("1234").is_err());
}

pub fn check_doi(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^10.\d{3,6}/\S+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "DOI (expected, eg, '10.1234/aksjdfh')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_doi() {
    assert!(check_doi("10.1234/aksjdfh").is_ok());
    assert!(check_doi("10.1234/ak../2949_-d.(asdf)fh").is_ok());
    assert!(check_doi("10.1234/ßs").is_err());
    assert!(check_doi("10.1234/aksjdfh ").is_err());
    assert!(check_doi("10.1234/ak sjdfh").is_err());
    assert!(check_doi("10.1234/aks\tjdfh").is_err());
    assert!(check_doi("10.1234/ ").is_err());
    assert!(check_doi("10.2/aksjdfh").is_err());
    assert!(check_doi("10.1234/\naksjdfh").is_err());
    assert!(check_doi("10.1234").is_err());
    assert!(check_doi("10.1234/").is_err());
}

pub fn check_arxiv_id(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^(\d{4}.\d{4,5}|[a-z\-]+(\.[A-Z]{2})?/\d{7})v\d+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "versioned arXiv identifier (expected, eg, '0806.2878v1')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_arxiv_id() {
    assert!(check_arxiv_id("0806.2878v1").is_ok());
    assert!(check_arxiv_id("1501.00001v1").is_ok());
    assert!(check_arxiv_id("hep-th/9901001v1").is_ok());
    assert!(check_arxiv_id("math.CA/0611800v2").is_ok());

    assert!(check_arxiv_id("hep-TH/9901001v1").is_err());
    assert!(check_arxiv_id("hßp-th/9901001v1").is_err());
    assert!(check_arxiv_id("math.CA/06l1800v2").is_err());
    assert!(check_arxiv_id("mßth.ca/0611800v2").is_err());
    assert!(check_arxiv_id("MATH.CA/0611800v2").is_err());
    assert!(check_arxiv_id("0806.2878v23").is_ok());
    assert!(check_arxiv_id("0806.2878v").is_err());
    assert!(check_arxiv_id("0806.2878").is_err());
    assert!(check_arxiv_id("0806.2878v1 ").is_err());
    assert!(check_arxiv_id("006.2878v1").is_err());
    assert!(check_arxiv_id("0806.v1").is_err());
    assert!(check_arxiv_id("08062878v1").is_err());
}

pub fn check_ark_id(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^ark:/\d{5,9}/\S+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "ARK identifier (expected, eg, 'ark:/13030/m53r5pzm')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_ark_id() {
    assert!(check_ark_id("ark:/13030/m53r5pzm").is_ok());
    assert!(check_ark_id("ark:/13030/m53r5pzm ").is_err());
    assert!(check_ark_id("ark:/13030/m53r5ßzm").is_err());
    assert!(check_ark_id("ARK:/13030/m53r5pzm").is_err());
    assert!(check_ark_id("ark:/13030/m53r5pzm.bla-deedah").is_ok());
    assert!(check_ark_id("/13030/m53r5pzm").is_err());
    assert!(check_ark_id("ark:/blah/m53r5pzm").is_err());
    assert!(check_ark_id("ark:/13030/").is_err());
    assert!(check_ark_id("ark:/13030").is_err());
}

pub fn check_isbn13(raw: &str) -> Result<()> {
    lazy_static! {
        // via https://stackoverflow.com/a/4381556
        static ref RE: Regex = Regex::new(r"^97(?:8|9)-\d{1,5}-\d{1,7}-\d{1,6}-\d$").unwrap();
    }
    if raw.len() == 17 && raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "Canonical ISBN-13 (expected, eg, '978-1-56619-909-4')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_isbn13() {
    assert!(check_isbn13("978-1-56619-909-4").is_ok());
    assert!(check_isbn13("978-1-4028-9462-6").is_ok());
    assert!(check_isbn13("978-1-56619-909-4 ").is_err());
    assert!(check_isbn13("9781566199094").is_err());
}

pub fn check_doaj_id(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-f0-9]{32}$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedChecksum(
            "DOAJ Article Identifier (expected, eg, 'e58f08a11ecb495ead55a44ad4f89808')"
                .to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_doaj_id() {
    assert!(check_doaj_id("e58f08a11ecb495ead55a44ad4f89808").is_ok());
    assert!(check_doaj_id("1b39813549077b2347c0f370c3864b40").is_ok());
    assert!(check_doaj_id("1b39813549077b2347c0f370c3864b40 ").is_err());
    assert!(check_doaj_id("1g39813549077b2347c0f370c3864b40").is_err());
    assert!(check_doaj_id("1B39813549077B2347C0F370c3864b40").is_err());
    assert!(check_doaj_id("1b39813549077b2347c0f370c3864b4").is_err());
    assert!(check_doaj_id("1b39813549077b2347c0f370c3864b411").is_err());
}

pub fn check_dblp_id(raw: &str) -> Result<()> {
    lazy_static! {
        // TODO: what should this actually be? more or less restrictive?
        static ref RE: Regex = Regex::new(r"^[a-z]+/[a-zA-Z0-9]+/[a-zA-Z0-9/]+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedChecksum(
            "dblp Article Key (expected, eg, 'journals/entcs/GoubaultM12')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_dblp_id() {
    assert!(check_dblp_id("journals/entcs/GoubaultM12").is_ok());
    assert!(check_dblp_id("journals/entcs/GoubaultM12").is_ok());
    assert!(check_dblp_id("10.123*").is_err());
    assert!(check_dblp_id("").is_err());
}

pub fn check_oai_id(raw: &str) -> Result<()> {
    lazy_static! {
        // http://www.openarchives.org/OAI/2.0/guidelines-oai-identifier.htm
        static ref RE: Regex = Regex::new(r"^oai:[a-zA-Z][a-zA-Z0-9\-]*(\.[a-zA-Z][a-zA-Z0-9\-]*)+:[a-zA-Z0-9\-_\.!~\*'\(\);/\?:@&=\+$,%]+$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedChecksum(
            "OAI-PMH identifier (expected, eg, 'oai:foo.org:some-local-id-54')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_oai_id() {
    assert!(check_oai_id("journals/entcs/GoubaultM12").is_err());
    assert!(check_oai_id("10.123*").is_err());
    assert!(check_oai_id("").is_err());
    assert!(check_oai_id("something:arXiv.org:hep-th/9901001").is_err()); // bad schema
    assert!(check_oai_id("oai:999:abc123").is_err()); // namespace-identifier must not start with digit
    assert!(check_oai_id("oai:wibble:abc123").is_err()); // namespace-identifier must be domain name
    assert!(check_oai_id("oai:wibble.org:ab cd").is_err()); // space not permitted (must be escaped as %20)
    assert!(check_oai_id("oai:wibble.org:ab#cd").is_err()); // # not permitted
    assert!(check_oai_id("oai:wibble.org:ab<cd").is_err()); // < not permitted
    // the "official" regex used above allows this case
    //assert!(check_oai_id("oai:wibble.org:ab%3ccd").is_err()); // < must be escaped at %3C not %3c

    assert!(check_oai_id("oai:arXiv.org:hep-th/9901001").is_ok());
    assert!(check_oai_id("oai:foo.org:some-local-id-53").is_ok());
    assert!(check_oai_id("oai:FOO.ORG:some-local-id-53").is_ok());
    assert!(check_oai_id("oai:foo.org:some-local-id-54").is_ok());
    assert!(check_oai_id("oai:foo.org:Some-Local-Id-54").is_ok());
    assert!(check_oai_id("oai:wibble.org:ab%20cd").is_ok());
    assert!(check_oai_id("oai:wibble.org:ab?cd").is_ok());
}

pub fn check_issn(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d{4}-\d{3}[0-9X]$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
        Ok(())
    } else {
        Err(FatcatError::MalformedExternalId(
            "ISSN (expected, eg, '1234-5678')".to_string(),
            raw.to_string(),
        ))?
    }
}

#[test]
fn test_check_issn() {
    assert!(check_issn("1234-5678").is_ok());
    assert!(check_issn("1234-567X").is_ok());
    assert!(check_issn("1234-5678 ").is_err());
    assert!(check_issn(" 1234-5678").is_err());
    assert!(check_issn("12345678").is_err());
    assert!(check_issn("0123-56789").is_err());
}

pub fn check_orcid(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^\d{4}-\d{4}-\d{4}-\d{3}[\dX]$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
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
    assert!(check_orcid("0123-4567-3456-6789 ").is_err());
    assert!(check_orcid("01234567-3456-6780").is_err());
    assert!(check_orcid("0x23-4567-3456-6780").is_err());
}

pub fn check_md5(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-f0-9]{32}$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
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
    assert!(check_md5("1b39813549077b2347c0f370c3864b40 ").is_err());
    assert!(check_md5("1g39813549077b2347c0f370c3864b40").is_err());
    assert!(check_md5("1B39813549077B2347C0F370c3864b40").is_err());
    assert!(check_md5("1b39813549077b2347c0f370c3864b4").is_err());
    assert!(check_md5("1b39813549077b2347c0f370c3864b411").is_err());
}

pub fn check_sha1(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-f0-9]{40}$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
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
    assert!(check_sha1("e9dd75237c94b209dc3ccd52722de6931a310ba3 ").is_err());
    assert!(check_sha1("g9dd75237c94b209dc3ccd52722de6931a310ba3").is_err());
    assert!(check_sha1("e9DD75237C94B209DC3CCD52722de6931a310ba3").is_err());
    assert!(check_sha1("e9dd75237c94b209dc3ccd52722de6931a310ba").is_err());
    assert!(check_sha1("e9dd75237c94b209dc3ccd52722de6931a310ba33").is_err());
}

pub fn check_sha256(raw: &str) -> Result<()> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^[a-f0-9]{64}$").unwrap();
    }
    if raw.is_ascii() && RE.is_match(raw) {
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
        check_sha256("cb1c378f464d5935ddaa8de28446d82638396c61f042295d7fb85e3cccc9e452 ").is_err()
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
        "retraction",
        "component",
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
    assert!(check_release_type("retraction").is_ok());
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

pub fn check_withdrawn_status(raw: &str) -> Result<()> {
    let valid_types = vec![
        // Didn't have a controlled vocab so made one up
        "withdrawn", // generic
        "retracted",
        "concern",
        "legal",
        "safety",
        "national-security",
        "spam",
        // potential more-specific statuses
        //"author-misconduct",
        //"author-mistake",
        //"publisher-misconduct",
        //"publisher-mistake",
        //"unreliable",
    ];
    for good in valid_types {
        if raw == good {
            return Ok(());
        }
    }
    Err(FatcatError::NotInControlledVocabulary(
        "withdrawn_status (controlled vocabulary)".to_string(),
        raw.to_string(),
    ))?
}

#[test]
fn test_check_withdrawn_status() {
    assert!(check_withdrawn_status("legal").is_ok());
    assert!(check_withdrawn_status("withdrawn").is_ok());
    assert!(check_withdrawn_status("boondogle").is_err());
    assert!(check_withdrawn_status("").is_err());
    assert!(check_withdrawn_status("editor ").is_err());
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
