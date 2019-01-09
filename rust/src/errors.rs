//! Crate-specific Result, Error, and ErrorKind types (using `error_chain`)

error_chain! {
    foreign_links { Fmt(::std::fmt::Error);
                    Diesel(::diesel::result::Error);
                    R2d2(::diesel::r2d2::Error);
                    Uuid(::uuid::ParseError);
                    Io(::std::io::Error) #[cfg(unix)];
                    Serde(::serde_json::Error);
                    Utf8Decode(::std::string::FromUtf8Error);
                    StringDecode(::data_encoding::DecodeError);
    }
    errors {
        InvalidFatcatId(id: String) {
            description("invalid fatcat identifier syntax")
            display("invalid fatcat identifier (expect 26-char base32 encoded): {}", id)
        }
        MalformedExternalId(id: String) {
            description("external identifier doesn't match required pattern")
            display("external identifier doesn't match required pattern: {}", id)
        }
        MalformedChecksum(hash: String) {
            description("checksum doesn't match required pattern (hex encoding)")
            display("checksum doesn't match required pattern (hex encoding): {}", hash)
        }
        NotInControlledVocabulary(word: String) {
            description("word or type not correct for controlled vocabulary")
            display("word or type not correct for controlled vocabulary")
        }
        EditgroupAlreadyAccepted(id: String) {
            description("editgroup was already accepted")
            display("attempted to accept or mutate an editgroup which was already accepted: {}", id)
        }
        MissingOrMultipleExternalId(message: String) {
            description("external identifiers missing or multiple specified")
            display("external identifiers missing or multiple specified; please supply exactly one")
        }
        InvalidEntityStateTransform(message: String) {
            description("Invalid Entity State Transform")
            display("tried to mutate an entity which was not in an appropriate state: {}", message)
        }
        InvalidCredentials(message: String) {
            description("auth token was missing, expired, revoked, or corrupt")
            display("auth token was missing, expired, revoked, or corrupt: {}", message)
        }
        InsufficientPrivileges(message: String) {
            description("editor account doesn't have authorization")
            display("editor account doesn't have authorization: {}", message)
        }
        OtherBadRequest(message: String) {
            description("catch-all error for bad or unallowed requests")
            display("broke a constraint or made an otherwise invalid request: {}", message)
        }
    }
}
