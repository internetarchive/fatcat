#![allow(unused_qualifications)]

#[cfg(any(feature = "client", feature = "server"))]
use crate::header;
use crate::models;

// Methods for converting between header::IntoHeaderValue<AuthOidc> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AuthOidc>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AuthOidc>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AuthOidc - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AuthOidc> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AuthOidc as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AuthOidc - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AuthOidc {
    /// Fatcat-specific short name (slug) for remote service being used for authentication.
    #[serde(rename = "provider")]
    pub provider: String,

    /// `SUB` from OIDC protocol. Usually a URL.
    #[serde(rename = "sub")]
    pub sub: String,

    /// `ISS` from OIDC protocol. Usually a stable account username, number, or identifier.
    #[serde(rename = "iss")]
    pub iss: String,

    /// What it sounds like; returned by OIDC, and used as a hint when creating new editor accounts. Fatcat usernames are usually this string with the `provider` slug as a suffix, though some munging may occur.
    #[serde(rename = "preferred_username")]
    pub preferred_username: String,
}

impl AuthOidc {
    pub fn new(provider: String, sub: String, iss: String, preferred_username: String) -> AuthOidc {
        AuthOidc {
            provider: provider,
            sub: sub,
            iss: iss,
            preferred_username: preferred_username,
        }
    }
}

/// Converts the AuthOidc value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AuthOidc {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("provider".to_string());
        params.push(self.provider.to_string());

        params.push("sub".to_string());
        params.push(self.sub.to_string());

        params.push("iss".to_string());
        params.push(self.iss.to_string());

        params.push("preferred_username".to_string());
        params.push(self.preferred_username.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AuthOidc value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AuthOidc {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub provider: Vec<String>,
            pub sub: Vec<String>,
            pub iss: Vec<String>,
            pub preferred_username: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AuthOidc".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "provider" => intermediate_rep
                        .provider
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "sub" => intermediate_rep
                        .sub
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "iss" => intermediate_rep
                        .iss
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "preferred_username" => intermediate_rep
                        .preferred_username
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AuthOidc".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AuthOidc {
            provider: intermediate_rep
                .provider
                .into_iter()
                .next()
                .ok_or("provider missing in AuthOidc".to_string())?,
            sub: intermediate_rep
                .sub
                .into_iter()
                .next()
                .ok_or("sub missing in AuthOidc".to_string())?,
            iss: intermediate_rep
                .iss
                .into_iter()
                .next()
                .ok_or("iss missing in AuthOidc".to_string())?,
            preferred_username: intermediate_rep
                .preferred_username
                .into_iter()
                .next()
                .ok_or("preferred_username missing in AuthOidc".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AuthOidcResult> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AuthOidcResult>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AuthOidcResult>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AuthOidcResult - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<AuthOidcResult> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AuthOidcResult as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AuthOidcResult - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AuthOidcResult {
    #[serde(rename = "editor")]
    pub editor: models::Editor,

    #[serde(rename = "token")]
    pub token: String,
}

impl AuthOidcResult {
    pub fn new(editor: models::Editor, token: String) -> AuthOidcResult {
        AuthOidcResult {
            editor: editor,
            token: token,
        }
    }
}

/// Converts the AuthOidcResult value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AuthOidcResult {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editor in query parameter serialization

        params.push("token".to_string());
        params.push(self.token.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AuthOidcResult value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AuthOidcResult {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editor: Vec<models::Editor>,
            pub token: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AuthOidcResult".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editor" => intermediate_rep
                        .editor
                        .push(models::Editor::from_str(val).map_err(|x| format!("{}", x))?),
                    "token" => intermediate_rep
                        .token
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AuthOidcResult".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AuthOidcResult {
            editor: intermediate_rep
                .editor
                .into_iter()
                .next()
                .ok_or("editor missing in AuthOidcResult".to_string())?,
            token: intermediate_rep
                .token
                .into_iter()
                .next()
                .ok_or("token missing in AuthOidcResult".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AuthTokenResult> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<AuthTokenResult>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<AuthTokenResult>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for AuthTokenResult - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<AuthTokenResult>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <AuthTokenResult as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into AuthTokenResult - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AuthTokenResult {
    #[serde(rename = "token")]
    pub token: String,
}

impl AuthTokenResult {
    pub fn new(token: String) -> AuthTokenResult {
        AuthTokenResult { token: token }
    }
}

/// Converts the AuthTokenResult value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for AuthTokenResult {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("token".to_string());
        params.push(self.token.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AuthTokenResult value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AuthTokenResult {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub token: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing AuthTokenResult".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "token" => intermediate_rep
                        .token
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing AuthTokenResult".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AuthTokenResult {
            token: intermediate_rep
                .token
                .into_iter()
                .next()
                .ok_or("token missing in AuthTokenResult".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ChangelogEntry> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ChangelogEntry>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ChangelogEntry>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ChangelogEntry - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ChangelogEntry> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ChangelogEntry as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ChangelogEntry - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ChangelogEntry {
    /// Monotonically increasing sequence number of this changelog entry.
    #[serde(rename = "index")]
    pub index: i64,

    /// Identifier of editgroup accepted/merged in this changelog entry.
    #[serde(rename = "editgroup_id")]
    pub editgroup_id: String,

    /// Date and time when the editgroup was accpeted.
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,

    #[serde(rename = "editgroup")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup: Option<models::Editgroup>,
}

impl ChangelogEntry {
    pub fn new(
        index: i64,
        editgroup_id: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> ChangelogEntry {
        ChangelogEntry {
            index: index,
            editgroup_id: editgroup_id,
            timestamp: timestamp,
            editgroup: None,
        }
    }
}

/// Converts the ChangelogEntry value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ChangelogEntry {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("index".to_string());
        params.push(self.index.to_string());

        params.push("editgroup_id".to_string());
        params.push(self.editgroup_id.to_string());

        // Skipping timestamp in query parameter serialization

        // Skipping editgroup in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ChangelogEntry value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ChangelogEntry {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub index: Vec<i64>,
            pub editgroup_id: Vec<String>,
            pub timestamp: Vec<chrono::DateTime<chrono::Utc>>,
            pub editgroup: Vec<models::Editgroup>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ChangelogEntry".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "index" => intermediate_rep
                        .index
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "editgroup_id" => intermediate_rep
                        .editgroup_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "timestamp" => intermediate_rep.timestamp.push(
                        chrono::DateTime::<chrono::Utc>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ChangelogEntry".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ChangelogEntry {
            index: intermediate_rep
                .index
                .into_iter()
                .next()
                .ok_or("index missing in ChangelogEntry".to_string())?,
            editgroup_id: intermediate_rep
                .editgroup_id
                .into_iter()
                .next()
                .ok_or("editgroup_id missing in ChangelogEntry".to_string())?,
            timestamp: intermediate_rep
                .timestamp
                .into_iter()
                .next()
                .ok_or("timestamp missing in ChangelogEntry".to_string())?,
            editgroup: intermediate_rep.editgroup.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ContainerAutoBatch> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ContainerAutoBatch>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ContainerAutoBatch>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ContainerAutoBatch - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ContainerAutoBatch>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ContainerAutoBatch as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ContainerAutoBatch - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ContainerAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::ContainerEntity>,
}

impl ContainerAutoBatch {
    pub fn new(
        editgroup: models::Editgroup,
        entity_list: Vec<models::ContainerEntity>,
    ) -> ContainerAutoBatch {
        ContainerAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

/// Converts the ContainerAutoBatch value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ContainerAutoBatch {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editgroup in query parameter serialization

        // Skipping entity_list in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ContainerAutoBatch value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ContainerAutoBatch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup: Vec<models::Editgroup>,
            pub entity_list: Vec<Vec<models::ContainerEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ContainerAutoBatch".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "entity_list" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in ContainerAutoBatch"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ContainerAutoBatch".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ContainerAutoBatch {
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in ContainerAutoBatch".to_string())?,
            entity_list: intermediate_rep
                .entity_list
                .into_iter()
                .next()
                .ok_or("entity_list missing in ContainerAutoBatch".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ContainerEntity> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ContainerEntity>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ContainerEntity>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ContainerEntity - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ContainerEntity>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ContainerEntity as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ContainerEntity - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ContainerEntity {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Name of the container (eg, Journal title). Required for entity creation.
    #[serde(rename = "name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Type of container, eg 'journal' or 'proceedings'. See Guide for list of valid types.
    #[serde(rename = "container_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_type: Option<String>,

    /// Name of the organization or entity responsible for publication. Not the complete imprint/brand.
    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Linking ISSN number (ISSN-L). Should be valid and registered with issn.org
    #[serde(rename = "issnl")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issnl: Option<String>,

    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,
}

impl ContainerEntity {
    pub fn new() -> ContainerEntity {
        ContainerEntity {
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
            name: None,
            container_type: None,
            publisher: None,
            issnl: None,
            wikidata_qid: None,
        }
    }
}

/// Converts the ContainerEntity value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ContainerEntity {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref state) = self.state {
            params.push("state".to_string());
            params.push(state.to_string());
        }

        if let Some(ref ident) = self.ident {
            params.push("ident".to_string());
            params.push(ident.to_string());
        }

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref redirect) = self.redirect {
            params.push("redirect".to_string());
            params.push(redirect.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping edit_extra in query parameter serialization
        // Skipping edit_extra in query parameter serialization

        if let Some(ref name) = self.name {
            params.push("name".to_string());
            params.push(name.to_string());
        }

        if let Some(ref container_type) = self.container_type {
            params.push("container_type".to_string());
            params.push(container_type.to_string());
        }

        if let Some(ref publisher) = self.publisher {
            params.push("publisher".to_string());
            params.push(publisher.to_string());
        }

        if let Some(ref issnl) = self.issnl {
            params.push("issnl".to_string());
            params.push(issnl.to_string());
        }

        if let Some(ref wikidata_qid) = self.wikidata_qid {
            params.push("wikidata_qid".to_string());
            params.push(wikidata_qid.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ContainerEntity value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ContainerEntity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub redirect: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub edit_extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub name: Vec<String>,
            pub container_type: Vec<String>,
            pub publisher: Vec<String>,
            pub issnl: Vec<String>,
            pub wikidata_qid: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ContainerEntity".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep
                        .state
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect" => intermediate_rep
                        .redirect
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ContainerEntity"
                                .to_string(),
                        )
                    }
                    "edit_extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ContainerEntity"
                                .to_string(),
                        )
                    }
                    "name" => intermediate_rep
                        .name
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "container_type" => intermediate_rep
                        .container_type
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "publisher" => intermediate_rep
                        .publisher
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "issnl" => intermediate_rep
                        .issnl
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "wikidata_qid" => intermediate_rep
                        .wikidata_qid
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ContainerEntity".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ContainerEntity {
            state: intermediate_rep.state.into_iter().next(),
            ident: intermediate_rep.ident.into_iter().next(),
            revision: intermediate_rep.revision.into_iter().next(),
            redirect: intermediate_rep.redirect.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            edit_extra: intermediate_rep.edit_extra.into_iter().next(),
            name: intermediate_rep.name.into_iter().next(),
            container_type: intermediate_rep.container_type.into_iter().next(),
            publisher: intermediate_rep.publisher.into_iter().next(),
            issnl: intermediate_rep.issnl.into_iter().next(),
            wikidata_qid: intermediate_rep.wikidata_qid.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CreatorAutoBatch> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<CreatorAutoBatch>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<CreatorAutoBatch>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for CreatorAutoBatch - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<CreatorAutoBatch>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <CreatorAutoBatch as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into CreatorAutoBatch - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CreatorAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::CreatorEntity>,
}

impl CreatorAutoBatch {
    pub fn new(
        editgroup: models::Editgroup,
        entity_list: Vec<models::CreatorEntity>,
    ) -> CreatorAutoBatch {
        CreatorAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

/// Converts the CreatorAutoBatch value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CreatorAutoBatch {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editgroup in query parameter serialization

        // Skipping entity_list in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CreatorAutoBatch value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CreatorAutoBatch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup: Vec<models::Editgroup>,
            pub entity_list: Vec<Vec<models::CreatorEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing CreatorAutoBatch".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "entity_list" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in CreatorAutoBatch"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing CreatorAutoBatch".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CreatorAutoBatch {
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in CreatorAutoBatch".to_string())?,
            entity_list: intermediate_rep
                .entity_list
                .into_iter()
                .next()
                .ok_or("entity_list missing in CreatorAutoBatch".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CreatorEntity> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<CreatorEntity>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<CreatorEntity>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for CreatorEntity - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<CreatorEntity> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <CreatorEntity as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into CreatorEntity - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CreatorEntity {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Name as should be displayed in web interface or in author lists (not index/sorted). Required for valid entities.
    #[serde(rename = "display_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// In English commonly the first name, but ordering is context and culture specific.
    #[serde(rename = "given_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// In English commonly the last, or family name, but ordering is context and culture specific.
    #[serde(rename = "surname")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,

    /// ORCiD (https://orcid.org) identifier
    #[serde(rename = "orcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orcid: Option<String>,

    /// Wikidata entity QID
    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,
}

impl CreatorEntity {
    pub fn new() -> CreatorEntity {
        CreatorEntity {
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
            display_name: None,
            given_name: None,
            surname: None,
            orcid: None,
            wikidata_qid: None,
        }
    }
}

/// Converts the CreatorEntity value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for CreatorEntity {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref state) = self.state {
            params.push("state".to_string());
            params.push(state.to_string());
        }

        if let Some(ref ident) = self.ident {
            params.push("ident".to_string());
            params.push(ident.to_string());
        }

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref redirect) = self.redirect {
            params.push("redirect".to_string());
            params.push(redirect.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping edit_extra in query parameter serialization
        // Skipping edit_extra in query parameter serialization

        if let Some(ref display_name) = self.display_name {
            params.push("display_name".to_string());
            params.push(display_name.to_string());
        }

        if let Some(ref given_name) = self.given_name {
            params.push("given_name".to_string());
            params.push(given_name.to_string());
        }

        if let Some(ref surname) = self.surname {
            params.push("surname".to_string());
            params.push(surname.to_string());
        }

        if let Some(ref orcid) = self.orcid {
            params.push("orcid".to_string());
            params.push(orcid.to_string());
        }

        if let Some(ref wikidata_qid) = self.wikidata_qid {
            params.push("wikidata_qid".to_string());
            params.push(wikidata_qid.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CreatorEntity value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CreatorEntity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub redirect: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub edit_extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub display_name: Vec<String>,
            pub given_name: Vec<String>,
            pub surname: Vec<String>,
            pub orcid: Vec<String>,
            pub wikidata_qid: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing CreatorEntity".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep
                        .state
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect" => intermediate_rep
                        .redirect
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in CreatorEntity"
                                .to_string(),
                        )
                    }
                    "edit_extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in CreatorEntity"
                                .to_string(),
                        )
                    }
                    "display_name" => intermediate_rep
                        .display_name
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "given_name" => intermediate_rep
                        .given_name
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "surname" => intermediate_rep
                        .surname
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "orcid" => intermediate_rep
                        .orcid
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "wikidata_qid" => intermediate_rep
                        .wikidata_qid
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing CreatorEntity".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CreatorEntity {
            state: intermediate_rep.state.into_iter().next(),
            ident: intermediate_rep.ident.into_iter().next(),
            revision: intermediate_rep.revision.into_iter().next(),
            redirect: intermediate_rep.redirect.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            edit_extra: intermediate_rep.edit_extra.into_iter().next(),
            display_name: intermediate_rep.display_name.into_iter().next(),
            given_name: intermediate_rep.given_name.into_iter().next(),
            surname: intermediate_rep.surname.into_iter().next(),
            orcid: intermediate_rep.orcid.into_iter().next(),
            wikidata_qid: intermediate_rep.wikidata_qid.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Editgroup> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Editgroup>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Editgroup>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Editgroup - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Editgroup> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Editgroup as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Editgroup - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Editgroup {
    /// Fatcat identifier for this editgroup. Assigned on creation.
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    /// Fatcat identifer of editor that created this editgroup.
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    #[serde(rename = "editor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<models::Editor>,

    /// For accepted/merged editgroups, the changelog index that the accept occured at. WARNING: not populated in all contexts that an editgroup could be included in a response.
    #[serde(rename = "changelog_index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog_index: Option<i64>,

    /// Timestamp when this editgroup was first created.
    #[serde(rename = "created")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,

    /// Timestamp when this editgroup was most recently submitted for review. If withdrawn, or never submitted, will be `null`.
    #[serde(rename = "submitted")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted: Option<chrono::DateTime<chrono::Utc>>,

    /// Comment describing the changes in this editgroup. Can be updated with PUT request.
    #[serde(rename = "description")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Free-form JSON metadata attached to this editgroup. Eg, metadata provenance, or script user-agent details. See guide for (unenforced) schema norms.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Only included in GET responses, and not in all contexts. Do not include this field in PUT or POST requests.
    #[serde(rename = "annotations")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<Vec<models::EditgroupAnnotation>>,

    #[serde(rename = "edits")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edits: Option<models::EditgroupEdits>,
}

impl Editgroup {
    pub fn new() -> Editgroup {
        Editgroup {
            editgroup_id: None,
            editor_id: None,
            editor: None,
            changelog_index: None,
            created: None,
            submitted: None,
            description: None,
            extra: None,
            annotations: None,
            edits: None,
        }
    }
}

/// Converts the Editgroup value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Editgroup {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref editgroup_id) = self.editgroup_id {
            params.push("editgroup_id".to_string());
            params.push(editgroup_id.to_string());
        }

        if let Some(ref editor_id) = self.editor_id {
            params.push("editor_id".to_string());
            params.push(editor_id.to_string());
        }

        // Skipping editor in query parameter serialization

        if let Some(ref changelog_index) = self.changelog_index {
            params.push("changelog_index".to_string());
            params.push(changelog_index.to_string());
        }

        // Skipping created in query parameter serialization

        // Skipping submitted in query parameter serialization

        if let Some(ref description) = self.description {
            params.push("description".to_string());
            params.push(description.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping annotations in query parameter serialization

        // Skipping edits in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Editgroup value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Editgroup {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup_id: Vec<String>,
            pub editor_id: Vec<String>,
            pub editor: Vec<models::Editor>,
            pub changelog_index: Vec<i64>,
            pub created: Vec<chrono::DateTime<chrono::Utc>>,
            pub submitted: Vec<chrono::DateTime<chrono::Utc>>,
            pub description: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub annotations: Vec<Vec<models::EditgroupAnnotation>>,
            pub edits: Vec<models::EditgroupEdits>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Editgroup".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup_id" => intermediate_rep
                        .editgroup_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "editor_id" => intermediate_rep
                        .editor_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "editor" => intermediate_rep
                        .editor
                        .push(models::Editor::from_str(val).map_err(|x| format!("{}", x))?),
                    "changelog_index" => intermediate_rep
                        .changelog_index
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "created" => intermediate_rep.created.push(
                        chrono::DateTime::<chrono::Utc>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "submitted" => intermediate_rep.submitted.push(
                        chrono::DateTime::<chrono::Utc>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "description" => intermediate_rep
                        .description
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Editgroup"
                                .to_string(),
                        )
                    }
                    "annotations" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in Editgroup"
                                .to_string(),
                        )
                    }
                    "edits" => intermediate_rep
                        .edits
                        .push(models::EditgroupEdits::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Editgroup".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Editgroup {
            editgroup_id: intermediate_rep.editgroup_id.into_iter().next(),
            editor_id: intermediate_rep.editor_id.into_iter().next(),
            editor: intermediate_rep.editor.into_iter().next(),
            changelog_index: intermediate_rep.changelog_index.into_iter().next(),
            created: intermediate_rep.created.into_iter().next(),
            submitted: intermediate_rep.submitted.into_iter().next(),
            description: intermediate_rep.description.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            annotations: intermediate_rep.annotations.into_iter().next(),
            edits: intermediate_rep.edits.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EditgroupAnnotation> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<EditgroupAnnotation>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EditgroupAnnotation>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for EditgroupAnnotation - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<EditgroupAnnotation>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EditgroupAnnotation as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into EditgroupAnnotation - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EditgroupAnnotation {
    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "annotation_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation_id: Option<String>,

    /// Editgroup that this annotation applies to. Set automatically in creations based on URL parameter.
    #[serde(rename = "editgroup_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editgroup_id: Option<String>,

    /// Defaults to editor created the annotation via POST request.
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    #[serde(rename = "editor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<models::Editor>,

    /// Timestamp when annotation was first created.
    #[serde(rename = "created")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<chrono::DateTime<chrono::Utc>>,

    #[serde(rename = "comment_markdown")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_markdown: Option<String>,

    /// Additional free-form JSON metadata that can be included as part of the annotation (or even as the primary annotation itself). See guide for details.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl EditgroupAnnotation {
    pub fn new() -> EditgroupAnnotation {
        EditgroupAnnotation {
            annotation_id: None,
            editgroup_id: None,
            editor_id: None,
            editor: None,
            created: None,
            comment_markdown: None,
            extra: None,
        }
    }
}

/// Converts the EditgroupAnnotation value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for EditgroupAnnotation {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref annotation_id) = self.annotation_id {
            params.push("annotation_id".to_string());
            params.push(annotation_id.to_string());
        }

        if let Some(ref editgroup_id) = self.editgroup_id {
            params.push("editgroup_id".to_string());
            params.push(editgroup_id.to_string());
        }

        if let Some(ref editor_id) = self.editor_id {
            params.push("editor_id".to_string());
            params.push(editor_id.to_string());
        }

        // Skipping editor in query parameter serialization

        // Skipping created in query parameter serialization

        if let Some(ref comment_markdown) = self.comment_markdown {
            params.push("comment_markdown".to_string());
            params.push(comment_markdown.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EditgroupAnnotation value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EditgroupAnnotation {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub annotation_id: Vec<String>,
            pub editgroup_id: Vec<String>,
            pub editor_id: Vec<String>,
            pub editor: Vec<models::Editor>,
            pub created: Vec<chrono::DateTime<chrono::Utc>>,
            pub comment_markdown: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EditgroupAnnotation".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "annotation_id" => intermediate_rep
                        .annotation_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "editgroup_id" => intermediate_rep
                        .editgroup_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "editor_id" => intermediate_rep
                        .editor_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "editor" => intermediate_rep
                        .editor
                        .push(models::Editor::from_str(val).map_err(|x| format!("{}", x))?),
                    "created" => intermediate_rep.created.push(
                        chrono::DateTime::<chrono::Utc>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "comment_markdown" => intermediate_rep
                        .comment_markdown
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in EditgroupAnnotation"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing EditgroupAnnotation".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EditgroupAnnotation {
            annotation_id: intermediate_rep.annotation_id.into_iter().next(),
            editgroup_id: intermediate_rep.editgroup_id.into_iter().next(),
            editor_id: intermediate_rep.editor_id.into_iter().next(),
            editor: intermediate_rep.editor.into_iter().next(),
            created: intermediate_rep.created.into_iter().next(),
            comment_markdown: intermediate_rep.comment_markdown.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
        })
    }
}

/// Only included in GET responses, and not in all contexts. Do not include this field in PUT or POST requests.
// Methods for converting between header::IntoHeaderValue<EditgroupEdits> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<EditgroupEdits>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EditgroupEdits>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for EditgroupEdits - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<EditgroupEdits> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EditgroupEdits as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into EditgroupEdits - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EditgroupEdits {
    #[serde(rename = "containers")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub containers: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "creators")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creators: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "files")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "filesets")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesets: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "webcaptures")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webcaptures: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::EntityEdit>>,

    #[serde(rename = "works")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub works: Option<Vec<models::EntityEdit>>,
}

impl EditgroupEdits {
    pub fn new() -> EditgroupEdits {
        EditgroupEdits {
            containers: None,
            creators: None,
            files: None,
            filesets: None,
            webcaptures: None,
            releases: None,
            works: None,
        }
    }
}

/// Converts the EditgroupEdits value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for EditgroupEdits {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping containers in query parameter serialization

        // Skipping creators in query parameter serialization

        // Skipping files in query parameter serialization

        // Skipping filesets in query parameter serialization

        // Skipping webcaptures in query parameter serialization

        // Skipping releases in query parameter serialization

        // Skipping works in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EditgroupEdits value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EditgroupEdits {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub containers: Vec<Vec<models::EntityEdit>>,
            pub creators: Vec<Vec<models::EntityEdit>>,
            pub files: Vec<Vec<models::EntityEdit>>,
            pub filesets: Vec<Vec<models::EntityEdit>>,
            pub webcaptures: Vec<Vec<models::EntityEdit>>,
            pub releases: Vec<Vec<models::EntityEdit>>,
            pub works: Vec<Vec<models::EntityEdit>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EditgroupEdits".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "containers" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EditgroupEdits"
                                .to_string(),
                        )
                    }
                    "creators" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EditgroupEdits"
                                .to_string(),
                        )
                    }
                    "files" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EditgroupEdits"
                                .to_string(),
                        )
                    }
                    "filesets" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EditgroupEdits"
                                .to_string(),
                        )
                    }
                    "webcaptures" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EditgroupEdits"
                                .to_string(),
                        )
                    }
                    "releases" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EditgroupEdits"
                                .to_string(),
                        )
                    }
                    "works" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EditgroupEdits"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing EditgroupEdits".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EditgroupEdits {
            containers: intermediate_rep.containers.into_iter().next(),
            creators: intermediate_rep.creators.into_iter().next(),
            files: intermediate_rep.files.into_iter().next(),
            filesets: intermediate_rep.filesets.into_iter().next(),
            webcaptures: intermediate_rep.webcaptures.into_iter().next(),
            releases: intermediate_rep.releases.into_iter().next(),
            works: intermediate_rep.works.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Editor> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Editor>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Editor>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Editor - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Editor> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Editor as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Editor - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Editor {
    /// Fatcat identifier for the editor. Can not be changed.
    #[serde(rename = "editor_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_id: Option<String>,

    /// Username/handle (short slug-like string) to identify this editor. May be changed at any time by the editor; use the `editor_id` as a persistend identifer.
    #[serde(rename = "username")]
    pub username: String,

    /// Whether this editor has the `admin` role.
    #[serde(rename = "is_admin")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,

    /// Whether this editor is a bot (as opposed to a human making manual edits)
    #[serde(rename = "is_bot")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_bot: Option<bool>,

    /// Whether this editor's account is enabled (if not API tokens and web logins will not work).
    #[serde(rename = "is_active")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

impl Editor {
    pub fn new(username: String) -> Editor {
        Editor {
            editor_id: None,
            username: username,
            is_admin: None,
            is_bot: None,
            is_active: None,
        }
    }
}

/// Converts the Editor value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Editor {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref editor_id) = self.editor_id {
            params.push("editor_id".to_string());
            params.push(editor_id.to_string());
        }

        params.push("username".to_string());
        params.push(self.username.to_string());

        if let Some(ref is_admin) = self.is_admin {
            params.push("is_admin".to_string());
            params.push(is_admin.to_string());
        }

        if let Some(ref is_bot) = self.is_bot {
            params.push("is_bot".to_string());
            params.push(is_bot.to_string());
        }

        if let Some(ref is_active) = self.is_active {
            params.push("is_active".to_string());
            params.push(is_active.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Editor value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Editor {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editor_id: Vec<String>,
            pub username: Vec<String>,
            pub is_admin: Vec<bool>,
            pub is_bot: Vec<bool>,
            pub is_active: Vec<bool>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Editor".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editor_id" => intermediate_rep
                        .editor_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "username" => intermediate_rep
                        .username
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "is_admin" => intermediate_rep
                        .is_admin
                        .push(bool::from_str(val).map_err(|x| format!("{}", x))?),
                    "is_bot" => intermediate_rep
                        .is_bot
                        .push(bool::from_str(val).map_err(|x| format!("{}", x))?),
                    "is_active" => intermediate_rep
                        .is_active
                        .push(bool::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Editor".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Editor {
            editor_id: intermediate_rep.editor_id.into_iter().next(),
            username: intermediate_rep
                .username
                .into_iter()
                .next()
                .ok_or("username missing in Editor".to_string())?,
            is_admin: intermediate_rep.is_admin.into_iter().next(),
            is_bot: intermediate_rep.is_bot.into_iter().next(),
            is_active: intermediate_rep.is_active.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EntityEdit> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<EntityEdit>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EntityEdit>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for EntityEdit - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<EntityEdit> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EntityEdit as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into EntityEdit - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EntityEdit {
    /// Unique UUID for this specific edit object.
    #[serde(rename = "edit_id")]
    pub edit_id: String,

    /// Fatcat identifier of the entity this edit is mutating.
    #[serde(rename = "ident")]
    pub ident: String,

    /// Entity revision that this edit will set the entity to. May be `null` in the case of deletions.
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// Revision of entity just before this edit. May be used in the future to prevent edit race conditions.
    #[serde(rename = "prev_revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_revision: Option<String>,

    /// When an edit is to merge entities (redirect one to another), this is the entity fatcat identifier for the target entity.
    #[serde(rename = "redirect_ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_ident: Option<String>,

    /// Editgroup identifier that this edit is part of.
    #[serde(rename = "editgroup_id")]
    pub editgroup_id: String,

    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl EntityEdit {
    pub fn new(edit_id: String, ident: String, editgroup_id: String) -> EntityEdit {
        EntityEdit {
            edit_id: edit_id,
            ident: ident,
            revision: None,
            prev_revision: None,
            redirect_ident: None,
            editgroup_id: editgroup_id,
            extra: None,
        }
    }
}

/// Converts the EntityEdit value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for EntityEdit {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("edit_id".to_string());
        params.push(self.edit_id.to_string());

        params.push("ident".to_string());
        params.push(self.ident.to_string());

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref prev_revision) = self.prev_revision {
            params.push("prev_revision".to_string());
            params.push(prev_revision.to_string());
        }

        if let Some(ref redirect_ident) = self.redirect_ident {
            params.push("redirect_ident".to_string());
            params.push(redirect_ident.to_string());
        }

        params.push("editgroup_id".to_string());
        params.push(self.editgroup_id.to_string());

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EntityEdit value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EntityEdit {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub edit_id: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub prev_revision: Vec<String>,
            pub redirect_ident: Vec<String>,
            pub editgroup_id: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EntityEdit".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "edit_id" => intermediate_rep
                        .edit_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "prev_revision" => intermediate_rep
                        .prev_revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect_ident" => intermediate_rep
                        .redirect_ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "editgroup_id" => intermediate_rep
                        .editgroup_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in EntityEdit"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing EntityEdit".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EntityEdit {
            edit_id: intermediate_rep
                .edit_id
                .into_iter()
                .next()
                .ok_or("edit_id missing in EntityEdit".to_string())?,
            ident: intermediate_rep
                .ident
                .into_iter()
                .next()
                .ok_or("ident missing in EntityEdit".to_string())?,
            revision: intermediate_rep.revision.into_iter().next(),
            prev_revision: intermediate_rep.prev_revision.into_iter().next(),
            redirect_ident: intermediate_rep.redirect_ident.into_iter().next(),
            editgroup_id: intermediate_rep
                .editgroup_id
                .into_iter()
                .next()
                .ok_or("editgroup_id missing in EntityEdit".to_string())?,
            extra: intermediate_rep.extra.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<EntityHistoryEntry> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<EntityHistoryEntry>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<EntityHistoryEntry>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for EntityHistoryEntry - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<EntityHistoryEntry>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <EntityHistoryEntry as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into EntityHistoryEntry - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct EntityHistoryEntry {
    #[serde(rename = "edit")]
    pub edit: models::EntityEdit,

    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "changelog_entry")]
    pub changelog_entry: models::ChangelogEntry,
}

impl EntityHistoryEntry {
    pub fn new(
        edit: models::EntityEdit,
        editgroup: models::Editgroup,
        changelog_entry: models::ChangelogEntry,
    ) -> EntityHistoryEntry {
        EntityHistoryEntry {
            edit: edit,
            editgroup: editgroup,
            changelog_entry: changelog_entry,
        }
    }
}

/// Converts the EntityHistoryEntry value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for EntityHistoryEntry {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping edit in query parameter serialization

        // Skipping editgroup in query parameter serialization

        // Skipping changelog_entry in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a EntityHistoryEntry value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for EntityHistoryEntry {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub edit: Vec<models::EntityEdit>,
            pub editgroup: Vec<models::Editgroup>,
            pub changelog_entry: Vec<models::ChangelogEntry>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing EntityHistoryEntry".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "edit" => intermediate_rep
                        .edit
                        .push(models::EntityEdit::from_str(val).map_err(|x| format!("{}", x))?),
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "changelog_entry" => intermediate_rep
                        .changelog_entry
                        .push(models::ChangelogEntry::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing EntityHistoryEntry".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(EntityHistoryEntry {
            edit: intermediate_rep
                .edit
                .into_iter()
                .next()
                .ok_or("edit missing in EntityHistoryEntry".to_string())?,
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in EntityHistoryEntry".to_string())?,
            changelog_entry: intermediate_rep
                .changelog_entry
                .into_iter()
                .next()
                .ok_or("changelog_entry missing in EntityHistoryEntry".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ErrorResponse> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ErrorResponse>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ErrorResponse>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ErrorResponse - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ErrorResponse> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ErrorResponse as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ErrorResponse - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ErrorResponse {
    #[serde(rename = "success")]
    pub success: bool,

    #[serde(rename = "error")]
    pub error: String,

    #[serde(rename = "message")]
    pub message: String,
}

impl ErrorResponse {
    pub fn new(success: bool, error: String, message: String) -> ErrorResponse {
        ErrorResponse {
            success: success,
            error: error,
            message: message,
        }
    }
}

/// Converts the ErrorResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ErrorResponse {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("success".to_string());
        params.push(self.success.to_string());

        params.push("error".to_string());
        params.push(self.error.to_string());

        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ErrorResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ErrorResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub success: Vec<bool>,
            pub error: Vec<String>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ErrorResponse".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "success" => intermediate_rep
                        .success
                        .push(bool::from_str(val).map_err(|x| format!("{}", x))?),
                    "error" => intermediate_rep
                        .error
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep
                        .message
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ErrorResponse".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ErrorResponse {
            success: intermediate_rep
                .success
                .into_iter()
                .next()
                .ok_or("success missing in ErrorResponse".to_string())?,
            error: intermediate_rep
                .error
                .into_iter()
                .next()
                .ok_or("error missing in ErrorResponse".to_string())?,
            message: intermediate_rep
                .message
                .into_iter()
                .next()
                .ok_or("message missing in ErrorResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FileAutoBatch> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<FileAutoBatch>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FileAutoBatch>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FileAutoBatch - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FileAutoBatch> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FileAutoBatch as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FileAutoBatch - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FileAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::FileEntity>,
}

impl FileAutoBatch {
    pub fn new(
        editgroup: models::Editgroup,
        entity_list: Vec<models::FileEntity>,
    ) -> FileAutoBatch {
        FileAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

/// Converts the FileAutoBatch value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FileAutoBatch {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editgroup in query parameter serialization

        // Skipping entity_list in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FileAutoBatch value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FileAutoBatch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup: Vec<models::Editgroup>,
            pub entity_list: Vec<Vec<models::FileEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FileAutoBatch".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "entity_list" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FileAutoBatch"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FileAutoBatch".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FileAutoBatch {
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in FileAutoBatch".to_string())?,
            entity_list: intermediate_rep
                .entity_list
                .into_iter()
                .next()
                .ok_or("entity_list missing in FileAutoBatch".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FileEntity> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<FileEntity>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FileEntity>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FileEntity - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FileEntity> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FileEntity as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FileEntity - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FileEntity {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Size of file in bytes. Non-zero.
    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// MD5 hash of data, in hex encoding
    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    /// SHA-256 hash of data, in hex encoding
    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    #[serde(rename = "urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<models::FileUrl>>,

    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// Set of identifier of release entities this file represents a full manifestation of. Usually a single release, but some files contain content of multiple full releases (eg, an issue of a journal).
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    /// Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.
    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::ReleaseEntity>>,
}

impl FileEntity {
    pub fn new() -> FileEntity {
        FileEntity {
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
            size: None,
            md5: None,
            sha1: None,
            sha256: None,
            urls: None,
            mimetype: None,
            release_ids: None,
            releases: None,
        }
    }
}

/// Converts the FileEntity value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FileEntity {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref state) = self.state {
            params.push("state".to_string());
            params.push(state.to_string());
        }

        if let Some(ref ident) = self.ident {
            params.push("ident".to_string());
            params.push(ident.to_string());
        }

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref redirect) = self.redirect {
            params.push("redirect".to_string());
            params.push(redirect.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping edit_extra in query parameter serialization
        // Skipping edit_extra in query parameter serialization

        if let Some(ref size) = self.size {
            params.push("size".to_string());
            params.push(size.to_string());
        }

        if let Some(ref md5) = self.md5 {
            params.push("md5".to_string());
            params.push(md5.to_string());
        }

        if let Some(ref sha1) = self.sha1 {
            params.push("sha1".to_string());
            params.push(sha1.to_string());
        }

        if let Some(ref sha256) = self.sha256 {
            params.push("sha256".to_string());
            params.push(sha256.to_string());
        }

        // Skipping urls in query parameter serialization

        if let Some(ref mimetype) = self.mimetype {
            params.push("mimetype".to_string());
            params.push(mimetype.to_string());
        }

        if let Some(ref release_ids) = self.release_ids {
            params.push("release_ids".to_string());
            params.push(
                release_ids
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping releases in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FileEntity value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FileEntity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub redirect: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub edit_extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub size: Vec<i64>,
            pub md5: Vec<String>,
            pub sha1: Vec<String>,
            pub sha256: Vec<String>,
            pub urls: Vec<Vec<models::FileUrl>>,
            pub mimetype: Vec<String>,
            pub release_ids: Vec<Vec<String>>,
            pub releases: Vec<Vec<models::ReleaseEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FileEntity".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep
                        .state
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect" => intermediate_rep
                        .redirect
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FileEntity"
                                .to_string(),
                        )
                    }
                    "edit_extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FileEntity"
                                .to_string(),
                        )
                    }
                    "size" => intermediate_rep
                        .size
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "md5" => intermediate_rep
                        .md5
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "sha1" => intermediate_rep
                        .sha1
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "sha256" => intermediate_rep
                        .sha256
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "urls" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FileEntity"
                                .to_string(),
                        )
                    }
                    "mimetype" => intermediate_rep
                        .mimetype
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "release_ids" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FileEntity"
                                .to_string(),
                        )
                    }
                    "releases" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FileEntity"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FileEntity".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FileEntity {
            state: intermediate_rep.state.into_iter().next(),
            ident: intermediate_rep.ident.into_iter().next(),
            revision: intermediate_rep.revision.into_iter().next(),
            redirect: intermediate_rep.redirect.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            edit_extra: intermediate_rep.edit_extra.into_iter().next(),
            size: intermediate_rep.size.into_iter().next(),
            md5: intermediate_rep.md5.into_iter().next(),
            sha1: intermediate_rep.sha1.into_iter().next(),
            sha256: intermediate_rep.sha256.into_iter().next(),
            urls: intermediate_rep.urls.into_iter().next(),
            mimetype: intermediate_rep.mimetype.into_iter().next(),
            release_ids: intermediate_rep.release_ids.into_iter().next(),
            releases: intermediate_rep.releases.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FileUrl> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<FileUrl>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FileUrl>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FileUrl - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FileUrl> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FileUrl as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FileUrl - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FileUrl {
    /// URL/URI pointing directly to a machine retrievable copy of this exact file.
    #[serde(rename = "url")]
    pub url: String,

    /// Indicates type of host this URL points to. Eg, \"publisher\", \"repository\", \"webarchive\". See guide for list of acceptable values.
    #[serde(rename = "rel")]
    pub rel: String,
}

impl FileUrl {
    pub fn new(url: String, rel: String) -> FileUrl {
        FileUrl { url: url, rel: rel }
    }
}

/// Converts the FileUrl value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FileUrl {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("url".to_string());
        params.push(self.url.to_string());

        params.push("rel".to_string());
        params.push(self.rel.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FileUrl value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FileUrl {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub url: Vec<String>,
            pub rel: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FileUrl".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "url" => intermediate_rep
                        .url
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "rel" => intermediate_rep
                        .rel
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FileUrl".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FileUrl {
            url: intermediate_rep
                .url
                .into_iter()
                .next()
                .ok_or("url missing in FileUrl".to_string())?,
            rel: intermediate_rep
                .rel
                .into_iter()
                .next()
                .ok_or("rel missing in FileUrl".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FilesetAutoBatch> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<FilesetAutoBatch>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FilesetAutoBatch>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FilesetAutoBatch - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<FilesetAutoBatch>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FilesetAutoBatch as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FilesetAutoBatch - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FilesetAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::FilesetEntity>,
}

impl FilesetAutoBatch {
    pub fn new(
        editgroup: models::Editgroup,
        entity_list: Vec<models::FilesetEntity>,
    ) -> FilesetAutoBatch {
        FilesetAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

/// Converts the FilesetAutoBatch value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FilesetAutoBatch {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editgroup in query parameter serialization

        // Skipping entity_list in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FilesetAutoBatch value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FilesetAutoBatch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup: Vec<models::Editgroup>,
            pub entity_list: Vec<Vec<models::FilesetEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FilesetAutoBatch".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "entity_list" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in FilesetAutoBatch"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FilesetAutoBatch".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FilesetAutoBatch {
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in FilesetAutoBatch".to_string())?,
            entity_list: intermediate_rep
                .entity_list
                .into_iter()
                .next()
                .ok_or("entity_list missing in FilesetAutoBatch".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FilesetEntity> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<FilesetEntity>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FilesetEntity>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FilesetEntity - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FilesetEntity> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FilesetEntity as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FilesetEntity - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FilesetEntity {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    #[serde(rename = "manifest")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest: Option<Vec<models::FilesetFile>>,

    #[serde(rename = "urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<models::FilesetUrl>>,

    /// Set of identifier of release entities this fileset represents a full manifestation of. Usually a single release.
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    /// Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.
    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::ReleaseEntity>>,
}

impl FilesetEntity {
    pub fn new() -> FilesetEntity {
        FilesetEntity {
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
            manifest: None,
            urls: None,
            release_ids: None,
            releases: None,
        }
    }
}

/// Converts the FilesetEntity value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FilesetEntity {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref state) = self.state {
            params.push("state".to_string());
            params.push(state.to_string());
        }

        if let Some(ref ident) = self.ident {
            params.push("ident".to_string());
            params.push(ident.to_string());
        }

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref redirect) = self.redirect {
            params.push("redirect".to_string());
            params.push(redirect.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping edit_extra in query parameter serialization
        // Skipping edit_extra in query parameter serialization

        // Skipping manifest in query parameter serialization

        // Skipping urls in query parameter serialization

        if let Some(ref release_ids) = self.release_ids {
            params.push("release_ids".to_string());
            params.push(
                release_ids
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping releases in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FilesetEntity value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FilesetEntity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub redirect: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub edit_extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub manifest: Vec<Vec<models::FilesetFile>>,
            pub urls: Vec<Vec<models::FilesetUrl>>,
            pub release_ids: Vec<Vec<String>>,
            pub releases: Vec<Vec<models::ReleaseEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FilesetEntity".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep
                        .state
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect" => intermediate_rep
                        .redirect
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FilesetEntity"
                                .to_string(),
                        )
                    }
                    "edit_extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FilesetEntity"
                                .to_string(),
                        )
                    }
                    "manifest" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FilesetEntity"
                                .to_string(),
                        )
                    }
                    "urls" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FilesetEntity"
                                .to_string(),
                        )
                    }
                    "release_ids" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FilesetEntity"
                                .to_string(),
                        )
                    }
                    "releases" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FilesetEntity"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FilesetEntity".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FilesetEntity {
            state: intermediate_rep.state.into_iter().next(),
            ident: intermediate_rep.ident.into_iter().next(),
            revision: intermediate_rep.revision.into_iter().next(),
            redirect: intermediate_rep.redirect.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            edit_extra: intermediate_rep.edit_extra.into_iter().next(),
            manifest: intermediate_rep.manifest.into_iter().next(),
            urls: intermediate_rep.urls.into_iter().next(),
            release_ids: intermediate_rep.release_ids.into_iter().next(),
            releases: intermediate_rep.releases.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FilesetFile> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<FilesetFile>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FilesetFile>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FilesetFile - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FilesetFile> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FilesetFile as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FilesetFile - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FilesetFile {
    /// Path name of file within this fileset (eg, directory)
    #[serde(rename = "path")]
    pub path: String,

    /// File size in bytes
    #[serde(rename = "size")]
    pub size: i64,

    /// MD5 hash of data, in hex encoding
    #[serde(rename = "md5")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5: Option<String>,

    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    /// SHA-256 hash of data, in hex encoding
    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,

    /// Free-form additional metadata about this specific file in the set. Eg, `mimetype`. See guide for nomative (but unenforced) schema fields.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl FilesetFile {
    pub fn new(path: String, size: i64) -> FilesetFile {
        FilesetFile {
            path: path,
            size: size,
            md5: None,
            sha1: None,
            sha256: None,
            extra: None,
        }
    }
}

/// Converts the FilesetFile value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FilesetFile {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("path".to_string());
        params.push(self.path.to_string());

        params.push("size".to_string());
        params.push(self.size.to_string());

        if let Some(ref md5) = self.md5 {
            params.push("md5".to_string());
            params.push(md5.to_string());
        }

        if let Some(ref sha1) = self.sha1 {
            params.push("sha1".to_string());
            params.push(sha1.to_string());
        }

        if let Some(ref sha256) = self.sha256 {
            params.push("sha256".to_string());
            params.push(sha256.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FilesetFile value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FilesetFile {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub path: Vec<String>,
            pub size: Vec<i64>,
            pub md5: Vec<String>,
            pub sha1: Vec<String>,
            pub sha256: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FilesetFile".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "path" => intermediate_rep
                        .path
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "size" => intermediate_rep
                        .size
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "md5" => intermediate_rep
                        .md5
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "sha1" => intermediate_rep
                        .sha1
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "sha256" => intermediate_rep
                        .sha256
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in FilesetFile"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FilesetFile".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FilesetFile {
            path: intermediate_rep
                .path
                .into_iter()
                .next()
                .ok_or("path missing in FilesetFile".to_string())?,
            size: intermediate_rep
                .size
                .into_iter()
                .next()
                .ok_or("size missing in FilesetFile".to_string())?,
            md5: intermediate_rep.md5.into_iter().next(),
            sha1: intermediate_rep.sha1.into_iter().next(),
            sha256: intermediate_rep.sha256.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<FilesetUrl> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<FilesetUrl>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<FilesetUrl>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for FilesetUrl - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<FilesetUrl> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <FilesetUrl as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into FilesetUrl - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct FilesetUrl {
    #[serde(rename = "url")]
    pub url: String,

    /// Indicates type of host this URL points to. See guide for list of acceptable values.
    #[serde(rename = "rel")]
    pub rel: String,
}

impl FilesetUrl {
    pub fn new(url: String, rel: String) -> FilesetUrl {
        FilesetUrl { url: url, rel: rel }
    }
}

/// Converts the FilesetUrl value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for FilesetUrl {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("url".to_string());
        params.push(self.url.to_string());

        params.push("rel".to_string());
        params.push(self.rel.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a FilesetUrl value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for FilesetUrl {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub url: Vec<String>,
            pub rel: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing FilesetUrl".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "url" => intermediate_rep
                        .url
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "rel" => intermediate_rep
                        .rel
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing FilesetUrl".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(FilesetUrl {
            url: intermediate_rep
                .url
                .into_iter()
                .next()
                .ok_or("url missing in FilesetUrl".to_string())?,
            rel: intermediate_rep
                .rel
                .into_iter()
                .next()
                .ok_or("rel missing in FilesetUrl".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReleaseAbstract> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ReleaseAbstract>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ReleaseAbstract>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ReleaseAbstract - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ReleaseAbstract>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ReleaseAbstract as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ReleaseAbstract - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReleaseAbstract {
    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    /// Abstract content. May be encoded, as per `mimetype` field, but only string/text content may be included.
    #[serde(rename = "content")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Mimetype of abstract contents. `text/plain` is the default if content isn't encoded.
    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// ISO language code of the abstract. Same semantics as release `language` field.
    #[serde(rename = "lang")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl ReleaseAbstract {
    pub fn new() -> ReleaseAbstract {
        ReleaseAbstract {
            sha1: None,
            content: None,
            mimetype: None,
            lang: None,
        }
    }
}

/// Converts the ReleaseAbstract value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ReleaseAbstract {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref sha1) = self.sha1 {
            params.push("sha1".to_string());
            params.push(sha1.to_string());
        }

        if let Some(ref content) = self.content {
            params.push("content".to_string());
            params.push(content.to_string());
        }

        if let Some(ref mimetype) = self.mimetype {
            params.push("mimetype".to_string());
            params.push(mimetype.to_string());
        }

        if let Some(ref lang) = self.lang {
            params.push("lang".to_string());
            params.push(lang.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReleaseAbstract value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReleaseAbstract {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub sha1: Vec<String>,
            pub content: Vec<String>,
            pub mimetype: Vec<String>,
            pub lang: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ReleaseAbstract".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "sha1" => intermediate_rep
                        .sha1
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "content" => intermediate_rep
                        .content
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "mimetype" => intermediate_rep
                        .mimetype
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "lang" => intermediate_rep
                        .lang
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ReleaseAbstract".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReleaseAbstract {
            sha1: intermediate_rep.sha1.into_iter().next(),
            content: intermediate_rep.content.into_iter().next(),
            mimetype: intermediate_rep.mimetype.into_iter().next(),
            lang: intermediate_rep.lang.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReleaseAutoBatch> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ReleaseAutoBatch>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ReleaseAutoBatch>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ReleaseAutoBatch - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<ReleaseAutoBatch>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ReleaseAutoBatch as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ReleaseAutoBatch - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReleaseAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::ReleaseEntity>,
}

impl ReleaseAutoBatch {
    pub fn new(
        editgroup: models::Editgroup,
        entity_list: Vec<models::ReleaseEntity>,
    ) -> ReleaseAutoBatch {
        ReleaseAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

/// Converts the ReleaseAutoBatch value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ReleaseAutoBatch {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editgroup in query parameter serialization

        // Skipping entity_list in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReleaseAutoBatch value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReleaseAutoBatch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup: Vec<models::Editgroup>,
            pub entity_list: Vec<Vec<models::ReleaseEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ReleaseAutoBatch".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "entity_list" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in ReleaseAutoBatch"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ReleaseAutoBatch".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReleaseAutoBatch {
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in ReleaseAutoBatch".to_string())?,
            entity_list: intermediate_rep
                .entity_list
                .into_iter()
                .next()
                .ok_or("entity_list missing in ReleaseAutoBatch".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReleaseContrib> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ReleaseContrib>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ReleaseContrib>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ReleaseContrib - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ReleaseContrib> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ReleaseContrib as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ReleaseContrib - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReleaseContrib {
    /// Internally assigned zero-indexed sequence number of contribution. Authors should come first; this encodes the order of attriubtion.
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// If known, indicates the creator entity this contribution was made by.
    #[serde(rename = "creator_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_id: Option<String>,

    #[serde(rename = "creator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<models::CreatorEntity>,

    /// Full name of the contributor as typeset in the release.
    #[serde(rename = "raw_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_name: Option<String>,

    /// In English commonly the first name, but ordering is context and culture specific.
    #[serde(rename = "given_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,

    /// In English commonly the last, or family name, but ordering is context and culture specific.
    #[serde(rename = "surname")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surname: Option<String>,

    /// Short string (slug) indicating type of contribution (eg, \"author\", \"translator\"). See guide for list of accpeted values.
    #[serde(rename = "role")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Raw affiliation string as displayed in text
    #[serde(rename = "raw_affiliation")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_affiliation: Option<String>,

    /// Additional free-form JSON metadata about this contributor/contribution. See guide for normative schema.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl ReleaseContrib {
    pub fn new() -> ReleaseContrib {
        ReleaseContrib {
            index: None,
            creator_id: None,
            creator: None,
            raw_name: None,
            given_name: None,
            surname: None,
            role: None,
            raw_affiliation: None,
            extra: None,
        }
    }
}

/// Converts the ReleaseContrib value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ReleaseContrib {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref index) = self.index {
            params.push("index".to_string());
            params.push(index.to_string());
        }

        if let Some(ref creator_id) = self.creator_id {
            params.push("creator_id".to_string());
            params.push(creator_id.to_string());
        }

        // Skipping creator in query parameter serialization

        if let Some(ref raw_name) = self.raw_name {
            params.push("raw_name".to_string());
            params.push(raw_name.to_string());
        }

        if let Some(ref given_name) = self.given_name {
            params.push("given_name".to_string());
            params.push(given_name.to_string());
        }

        if let Some(ref surname) = self.surname {
            params.push("surname".to_string());
            params.push(surname.to_string());
        }

        if let Some(ref role) = self.role {
            params.push("role".to_string());
            params.push(role.to_string());
        }

        if let Some(ref raw_affiliation) = self.raw_affiliation {
            params.push("raw_affiliation".to_string());
            params.push(raw_affiliation.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReleaseContrib value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReleaseContrib {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub index: Vec<i64>,
            pub creator_id: Vec<String>,
            pub creator: Vec<models::CreatorEntity>,
            pub raw_name: Vec<String>,
            pub given_name: Vec<String>,
            pub surname: Vec<String>,
            pub role: Vec<String>,
            pub raw_affiliation: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ReleaseContrib".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "index" => intermediate_rep
                        .index
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "creator_id" => intermediate_rep
                        .creator_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "creator" => intermediate_rep
                        .creator
                        .push(models::CreatorEntity::from_str(val).map_err(|x| format!("{}", x))?),
                    "raw_name" => intermediate_rep
                        .raw_name
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "given_name" => intermediate_rep
                        .given_name
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "surname" => intermediate_rep
                        .surname
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "role" => intermediate_rep
                        .role
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "raw_affiliation" => intermediate_rep
                        .raw_affiliation
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseContrib"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ReleaseContrib".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReleaseContrib {
            index: intermediate_rep.index.into_iter().next(),
            creator_id: intermediate_rep.creator_id.into_iter().next(),
            creator: intermediate_rep.creator.into_iter().next(),
            raw_name: intermediate_rep.raw_name.into_iter().next(),
            given_name: intermediate_rep.given_name.into_iter().next(),
            surname: intermediate_rep.surname.into_iter().next(),
            role: intermediate_rep.role.into_iter().next(),
            raw_affiliation: intermediate_rep.raw_affiliation.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReleaseEntity> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ReleaseEntity>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ReleaseEntity>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ReleaseEntity - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ReleaseEntity> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ReleaseEntity as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ReleaseEntity - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReleaseEntity {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Required for valid entities. The title used in citations and for display. Sometimes the English translation of title e even if release content is not English.
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Subtitle of release. In many cases, better to merge with title than include as separate field (unless combined title would be very long). See guide for details.
    #[serde(rename = "subtitle")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,

    /// Title in original language if `title` field has been translated. See guide for details.
    #[serde(rename = "original_title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_title: Option<String>,

    /// Identifier of work this release is part of. In creation (POST) requests, a work entity will be created automatically if this field is not set.
    #[serde(rename = "work_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_id: Option<String>,

    #[serde(rename = "container")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<models::ContainerEntity>,

    /// Complete file entities identified by `file_ids` field. Only included in GET responses when `files` included in `expand` parameter; ignored in PUT or POST requests.
    #[serde(rename = "files")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<models::FileEntity>>,

    /// Complete file entities identified by `filesets_ids` field. Only included in GET responses when `filesets` included in `expand` parameter; ignored in PUT or POST requests.
    #[serde(rename = "filesets")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesets: Option<Vec<models::FilesetEntity>>,

    /// Complete webcapture entities identified by `webcapture_ids` field. Only included in GET responses when `webcaptures` included in `expand` parameter; ignored in PUT or POST requests.
    #[serde(rename = "webcaptures")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webcaptures: Option<Vec<models::WebcaptureEntity>>,

    /// Used to link this release to a container entity that the release was published as part of.
    #[serde(rename = "container_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_id: Option<String>,

    /// \"Type\" or \"medium\" that this release is published as. See guide for valid values.
    #[serde(rename = "release_type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_type: Option<String>,

    /// The stage of publication of this specific release. See guide for valid values and semantics.
    #[serde(rename = "release_stage")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_stage: Option<String>,

    /// Full date when this release was formally published. ISO format, like `2019-03-05`. See guide for semantics.
    #[serde(rename = "release_date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_date: Option<chrono::NaiveDate>,

    /// Year when this release was formally published. Must match `release_date` if that field is set; this field exists because sometimes only the year is known.
    #[serde(rename = "release_year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_year: Option<i64>,

    /// Type of withdrawl or retraction of this release, if applicable. If release has not been withdrawn, should be `null` (aka, not set, not the string \"null\" or an empty string).
    #[serde(rename = "withdrawn_status")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_status: Option<String>,

    /// Full date when this release was formally withdrawn (if applicable). ISO format, like `release_date`.
    #[serde(rename = "withdrawn_date")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_date: Option<chrono::NaiveDate>,

    /// Year corresponding with `withdrawn_date` like `release_year`/`release_date`.
    #[serde(rename = "withdrawn_year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub withdrawn_year: Option<i64>,

    #[serde(rename = "ext_ids")]
    pub ext_ids: models::ReleaseExtIds,

    /// Volume number of container that this release was published in. Often corresponds to the \"Nth\" year of publication, but can be any string. See guide.
    #[serde(rename = "volume")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,

    /// Issue number of volume/container that this release was published in. Sometimes coresponds to a month number in the year, but can be any string. See guide.
    #[serde(rename = "issue")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<String>,

    /// Either a single page number (\"first page\") or a range of pages separated by a dash (\"-\"). See guide for details.
    #[serde(rename = "pages")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,

    /// For, eg, technical reports, which are published in series or assigned some other institutional or container-specific identifier.
    #[serde(rename = "number")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,

    /// For, eg, updated technical reports or software packages, where the version string may be the only field disambiguating between releases.
    #[serde(rename = "version")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Name, usually English, of the entity or institution responsible for publication of this release. Not necessarily the imprint/brand. See guide.
    #[serde(rename = "publisher")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Primary language of the content of the full release. Two-letter RFC1766/ISO639-1 language code, with some custom extensions/additions. See guide.
    #[serde(rename = "language")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// Short string (slug) name of license under which release is openly published (if applicable).
    #[serde(rename = "license_slug")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_slug: Option<String>,

    #[serde(rename = "contribs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contribs: Option<Vec<models::ReleaseContrib>>,

    #[serde(rename = "refs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refs: Option<Vec<models::ReleaseRef>>,

    #[serde(rename = "abstracts")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abstracts: Option<Vec<models::ReleaseAbstract>>,
}

impl ReleaseEntity {
    pub fn new(ext_ids: models::ReleaseExtIds) -> ReleaseEntity {
        ReleaseEntity {
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
            title: None,
            subtitle: None,
            original_title: None,
            work_id: None,
            container: None,
            files: None,
            filesets: None,
            webcaptures: None,
            container_id: None,
            release_type: None,
            release_stage: None,
            release_date: None,
            release_year: None,
            withdrawn_status: None,
            withdrawn_date: None,
            withdrawn_year: None,
            ext_ids: ext_ids,
            volume: None,
            issue: None,
            pages: None,
            number: None,
            version: None,
            publisher: None,
            language: None,
            license_slug: None,
            contribs: None,
            refs: None,
            abstracts: None,
        }
    }
}

/// Converts the ReleaseEntity value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ReleaseEntity {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref state) = self.state {
            params.push("state".to_string());
            params.push(state.to_string());
        }

        if let Some(ref ident) = self.ident {
            params.push("ident".to_string());
            params.push(ident.to_string());
        }

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref redirect) = self.redirect {
            params.push("redirect".to_string());
            params.push(redirect.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping edit_extra in query parameter serialization
        // Skipping edit_extra in query parameter serialization

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref subtitle) = self.subtitle {
            params.push("subtitle".to_string());
            params.push(subtitle.to_string());
        }

        if let Some(ref original_title) = self.original_title {
            params.push("original_title".to_string());
            params.push(original_title.to_string());
        }

        if let Some(ref work_id) = self.work_id {
            params.push("work_id".to_string());
            params.push(work_id.to_string());
        }

        // Skipping container in query parameter serialization

        // Skipping files in query parameter serialization

        // Skipping filesets in query parameter serialization

        // Skipping webcaptures in query parameter serialization

        if let Some(ref container_id) = self.container_id {
            params.push("container_id".to_string());
            params.push(container_id.to_string());
        }

        if let Some(ref release_type) = self.release_type {
            params.push("release_type".to_string());
            params.push(release_type.to_string());
        }

        if let Some(ref release_stage) = self.release_stage {
            params.push("release_stage".to_string());
            params.push(release_stage.to_string());
        }

        // Skipping release_date in query parameter serialization

        if let Some(ref release_year) = self.release_year {
            params.push("release_year".to_string());
            params.push(release_year.to_string());
        }

        if let Some(ref withdrawn_status) = self.withdrawn_status {
            params.push("withdrawn_status".to_string());
            params.push(withdrawn_status.to_string());
        }

        // Skipping withdrawn_date in query parameter serialization

        if let Some(ref withdrawn_year) = self.withdrawn_year {
            params.push("withdrawn_year".to_string());
            params.push(withdrawn_year.to_string());
        }

        // Skipping ext_ids in query parameter serialization

        if let Some(ref volume) = self.volume {
            params.push("volume".to_string());
            params.push(volume.to_string());
        }

        if let Some(ref issue) = self.issue {
            params.push("issue".to_string());
            params.push(issue.to_string());
        }

        if let Some(ref pages) = self.pages {
            params.push("pages".to_string());
            params.push(pages.to_string());
        }

        if let Some(ref number) = self.number {
            params.push("number".to_string());
            params.push(number.to_string());
        }

        if let Some(ref version) = self.version {
            params.push("version".to_string());
            params.push(version.to_string());
        }

        if let Some(ref publisher) = self.publisher {
            params.push("publisher".to_string());
            params.push(publisher.to_string());
        }

        if let Some(ref language) = self.language {
            params.push("language".to_string());
            params.push(language.to_string());
        }

        if let Some(ref license_slug) = self.license_slug {
            params.push("license_slug".to_string());
            params.push(license_slug.to_string());
        }

        // Skipping contribs in query parameter serialization

        // Skipping refs in query parameter serialization

        // Skipping abstracts in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReleaseEntity value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReleaseEntity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub redirect: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub edit_extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub title: Vec<String>,
            pub subtitle: Vec<String>,
            pub original_title: Vec<String>,
            pub work_id: Vec<String>,
            pub container: Vec<models::ContainerEntity>,
            pub files: Vec<Vec<models::FileEntity>>,
            pub filesets: Vec<Vec<models::FilesetEntity>>,
            pub webcaptures: Vec<Vec<models::WebcaptureEntity>>,
            pub container_id: Vec<String>,
            pub release_type: Vec<String>,
            pub release_stage: Vec<String>,
            pub release_date: Vec<chrono::NaiveDate>,
            pub release_year: Vec<i64>,
            pub withdrawn_status: Vec<String>,
            pub withdrawn_date: Vec<chrono::NaiveDate>,
            pub withdrawn_year: Vec<i64>,
            pub ext_ids: Vec<models::ReleaseExtIds>,
            pub volume: Vec<String>,
            pub issue: Vec<String>,
            pub pages: Vec<String>,
            pub number: Vec<String>,
            pub version: Vec<String>,
            pub publisher: Vec<String>,
            pub language: Vec<String>,
            pub license_slug: Vec<String>,
            pub contribs: Vec<Vec<models::ReleaseContrib>>,
            pub refs: Vec<Vec<models::ReleaseRef>>,
            pub abstracts: Vec<Vec<models::ReleaseAbstract>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ReleaseEntity".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep
                        .state
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect" => intermediate_rep
                        .redirect
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    "edit_extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    "title" => intermediate_rep
                        .title
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "subtitle" => intermediate_rep
                        .subtitle
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "original_title" => intermediate_rep
                        .original_title
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "work_id" => intermediate_rep
                        .work_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "container" => intermediate_rep.container.push(
                        models::ContainerEntity::from_str(val).map_err(|x| format!("{}", x))?,
                    ),
                    "files" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    "filesets" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    "webcaptures" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    "container_id" => intermediate_rep
                        .container_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "release_type" => intermediate_rep
                        .release_type
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "release_stage" => intermediate_rep
                        .release_stage
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "release_date" => intermediate_rep
                        .release_date
                        .push(chrono::NaiveDate::from_str(val).map_err(|x| format!("{}", x))?),
                    "release_year" => intermediate_rep
                        .release_year
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "withdrawn_status" => intermediate_rep
                        .withdrawn_status
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "withdrawn_date" => intermediate_rep
                        .withdrawn_date
                        .push(chrono::NaiveDate::from_str(val).map_err(|x| format!("{}", x))?),
                    "withdrawn_year" => intermediate_rep
                        .withdrawn_year
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "ext_ids" => intermediate_rep
                        .ext_ids
                        .push(models::ReleaseExtIds::from_str(val).map_err(|x| format!("{}", x))?),
                    "volume" => intermediate_rep
                        .volume
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "issue" => intermediate_rep
                        .issue
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "pages" => intermediate_rep
                        .pages
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "number" => intermediate_rep
                        .number
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "version" => intermediate_rep
                        .version
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "publisher" => intermediate_rep
                        .publisher
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "language" => intermediate_rep
                        .language
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "license_slug" => intermediate_rep
                        .license_slug
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "contribs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    "refs" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    "abstracts" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseEntity"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ReleaseEntity".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReleaseEntity {
            state: intermediate_rep.state.into_iter().next(),
            ident: intermediate_rep.ident.into_iter().next(),
            revision: intermediate_rep.revision.into_iter().next(),
            redirect: intermediate_rep.redirect.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            edit_extra: intermediate_rep.edit_extra.into_iter().next(),
            title: intermediate_rep.title.into_iter().next(),
            subtitle: intermediate_rep.subtitle.into_iter().next(),
            original_title: intermediate_rep.original_title.into_iter().next(),
            work_id: intermediate_rep.work_id.into_iter().next(),
            container: intermediate_rep.container.into_iter().next(),
            files: intermediate_rep.files.into_iter().next(),
            filesets: intermediate_rep.filesets.into_iter().next(),
            webcaptures: intermediate_rep.webcaptures.into_iter().next(),
            container_id: intermediate_rep.container_id.into_iter().next(),
            release_type: intermediate_rep.release_type.into_iter().next(),
            release_stage: intermediate_rep.release_stage.into_iter().next(),
            release_date: intermediate_rep.release_date.into_iter().next(),
            release_year: intermediate_rep.release_year.into_iter().next(),
            withdrawn_status: intermediate_rep.withdrawn_status.into_iter().next(),
            withdrawn_date: intermediate_rep.withdrawn_date.into_iter().next(),
            withdrawn_year: intermediate_rep.withdrawn_year.into_iter().next(),
            ext_ids: intermediate_rep
                .ext_ids
                .into_iter()
                .next()
                .ok_or("ext_ids missing in ReleaseEntity".to_string())?,
            volume: intermediate_rep.volume.into_iter().next(),
            issue: intermediate_rep.issue.into_iter().next(),
            pages: intermediate_rep.pages.into_iter().next(),
            number: intermediate_rep.number.into_iter().next(),
            version: intermediate_rep.version.into_iter().next(),
            publisher: intermediate_rep.publisher.into_iter().next(),
            language: intermediate_rep.language.into_iter().next(),
            license_slug: intermediate_rep.license_slug.into_iter().next(),
            contribs: intermediate_rep.contribs.into_iter().next(),
            refs: intermediate_rep.refs.into_iter().next(),
            abstracts: intermediate_rep.abstracts.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReleaseExtIds> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ReleaseExtIds>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ReleaseExtIds>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ReleaseExtIds - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ReleaseExtIds> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ReleaseExtIds as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ReleaseExtIds - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReleaseExtIds {
    /// Digital Object Identifier (DOI), mostly for published papers and datasets. Should be registered and resolvable via https://doi.org/
    #[serde(rename = "doi")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doi: Option<String>,

    /// Wikidata entity QID
    #[serde(rename = "wikidata_qid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wikidata_qid: Option<String>,

    /// ISBN-13, for books. Usually not set for chapters. ISBN-10 should be converted to ISBN-13.
    #[serde(rename = "isbn13")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isbn13: Option<String>,

    /// PubMed Identifier
    #[serde(rename = "pmid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmid: Option<String>,

    /// PubMed Central Identifier
    #[serde(rename = "pmcid")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmcid: Option<String>,

    /// CORE (https://core.ac.uk) identifier
    #[serde(rename = "core")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub core: Option<String>,

    /// arXiv (https://arxiv.org) identifier; must include version
    #[serde(rename = "arxiv")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arxiv: Option<String>,

    /// JSTOR work identifier
    #[serde(rename = "jstor")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jstor: Option<String>,

    /// ARK identifier
    #[serde(rename = "ark")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ark: Option<String>,

    /// Microsoft Academic Graph identifier
    #[serde(rename = "mag")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mag: Option<String>,
}

impl ReleaseExtIds {
    pub fn new() -> ReleaseExtIds {
        ReleaseExtIds {
            doi: None,
            wikidata_qid: None,
            isbn13: None,
            pmid: None,
            pmcid: None,
            core: None,
            arxiv: None,
            jstor: None,
            ark: None,
            mag: None,
        }
    }
}

/// Converts the ReleaseExtIds value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ReleaseExtIds {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref doi) = self.doi {
            params.push("doi".to_string());
            params.push(doi.to_string());
        }

        if let Some(ref wikidata_qid) = self.wikidata_qid {
            params.push("wikidata_qid".to_string());
            params.push(wikidata_qid.to_string());
        }

        if let Some(ref isbn13) = self.isbn13 {
            params.push("isbn13".to_string());
            params.push(isbn13.to_string());
        }

        if let Some(ref pmid) = self.pmid {
            params.push("pmid".to_string());
            params.push(pmid.to_string());
        }

        if let Some(ref pmcid) = self.pmcid {
            params.push("pmcid".to_string());
            params.push(pmcid.to_string());
        }

        if let Some(ref core) = self.core {
            params.push("core".to_string());
            params.push(core.to_string());
        }

        if let Some(ref arxiv) = self.arxiv {
            params.push("arxiv".to_string());
            params.push(arxiv.to_string());
        }

        if let Some(ref jstor) = self.jstor {
            params.push("jstor".to_string());
            params.push(jstor.to_string());
        }

        if let Some(ref ark) = self.ark {
            params.push("ark".to_string());
            params.push(ark.to_string());
        }

        if let Some(ref mag) = self.mag {
            params.push("mag".to_string());
            params.push(mag.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReleaseExtIds value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReleaseExtIds {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub doi: Vec<String>,
            pub wikidata_qid: Vec<String>,
            pub isbn13: Vec<String>,
            pub pmid: Vec<String>,
            pub pmcid: Vec<String>,
            pub core: Vec<String>,
            pub arxiv: Vec<String>,
            pub jstor: Vec<String>,
            pub ark: Vec<String>,
            pub mag: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ReleaseExtIds".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "doi" => intermediate_rep
                        .doi
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "wikidata_qid" => intermediate_rep
                        .wikidata_qid
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "isbn13" => intermediate_rep
                        .isbn13
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "pmid" => intermediate_rep
                        .pmid
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "pmcid" => intermediate_rep
                        .pmcid
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "core" => intermediate_rep
                        .core
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "arxiv" => intermediate_rep
                        .arxiv
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "jstor" => intermediate_rep
                        .jstor
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ark" => intermediate_rep
                        .ark
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "mag" => intermediate_rep
                        .mag
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ReleaseExtIds".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReleaseExtIds {
            doi: intermediate_rep.doi.into_iter().next(),
            wikidata_qid: intermediate_rep.wikidata_qid.into_iter().next(),
            isbn13: intermediate_rep.isbn13.into_iter().next(),
            pmid: intermediate_rep.pmid.into_iter().next(),
            pmcid: intermediate_rep.pmcid.into_iter().next(),
            core: intermediate_rep.core.into_iter().next(),
            arxiv: intermediate_rep.arxiv.into_iter().next(),
            jstor: intermediate_rep.jstor.into_iter().next(),
            ark: intermediate_rep.ark.into_iter().next(),
            mag: intermediate_rep.mag.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ReleaseRef> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<ReleaseRef>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<ReleaseRef>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for ReleaseRef - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<ReleaseRef> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <ReleaseRef as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into ReleaseRef - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ReleaseRef {
    /// Zero-indexed sequence number of this reference in the list of references. Assigned automatically and used internally; don't confuse with `key`.
    #[serde(rename = "index")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,

    /// Optional, fatcat identifier of release entity that this reference is citing.
    #[serde(rename = "target_release_id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_release_id: Option<String>,

    /// Additional free-form JSON metadata about this citation. Generally follows Citation Style Language (CSL) JSON schema. See guide for details.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Short string used to indicate this reference from within the release text; or numbering of references as typeset in the release itself. Optional; don't confuse with `index` field.
    #[serde(rename = "key")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Year that the cited work was published in.
    #[serde(rename = "year")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i64>,

    /// Name of the container (eg, journal) that the citation work was published as part of. May be an acronym or full name.
    #[serde(rename = "container_name")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,

    /// Name of the work being cited.
    #[serde(rename = "title")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Page number or other indicator of the specific subset of a work being cited. Not to be confused with the first page (or page range) of an entire paper or chapter being cited.
    #[serde(rename = "locator")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
}

impl ReleaseRef {
    pub fn new() -> ReleaseRef {
        ReleaseRef {
            index: None,
            target_release_id: None,
            extra: None,
            key: None,
            year: None,
            container_name: None,
            title: None,
            locator: None,
        }
    }
}

/// Converts the ReleaseRef value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for ReleaseRef {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref index) = self.index {
            params.push("index".to_string());
            params.push(index.to_string());
        }

        if let Some(ref target_release_id) = self.target_release_id {
            params.push("target_release_id".to_string());
            params.push(target_release_id.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        if let Some(ref key) = self.key {
            params.push("key".to_string());
            params.push(key.to_string());
        }

        if let Some(ref year) = self.year {
            params.push("year".to_string());
            params.push(year.to_string());
        }

        if let Some(ref container_name) = self.container_name {
            params.push("container_name".to_string());
            params.push(container_name.to_string());
        }

        if let Some(ref title) = self.title {
            params.push("title".to_string());
            params.push(title.to_string());
        }

        if let Some(ref locator) = self.locator {
            params.push("locator".to_string());
            params.push(locator.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ReleaseRef value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ReleaseRef {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub index: Vec<i64>,
            pub target_release_id: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub key: Vec<String>,
            pub year: Vec<i64>,
            pub container_name: Vec<String>,
            pub title: Vec<String>,
            pub locator: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing ReleaseRef".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "index" => intermediate_rep
                        .index
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "target_release_id" => intermediate_rep
                        .target_release_id
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in ReleaseRef"
                                .to_string(),
                        )
                    }
                    "key" => intermediate_rep
                        .key
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "year" => intermediate_rep
                        .year
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "container_name" => intermediate_rep
                        .container_name
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "title" => intermediate_rep
                        .title
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "locator" => intermediate_rep
                        .locator
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing ReleaseRef".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ReleaseRef {
            index: intermediate_rep.index.into_iter().next(),
            target_release_id: intermediate_rep.target_release_id.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            key: intermediate_rep.key.into_iter().next(),
            year: intermediate_rep.year.into_iter().next(),
            container_name: intermediate_rep.container_name.into_iter().next(),
            title: intermediate_rep.title.into_iter().next(),
            locator: intermediate_rep.locator.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Success> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<Success>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<Success>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for Success - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<Success> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <Success as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into Success - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Success {
    #[serde(rename = "success")]
    pub success: bool,

    #[serde(rename = "message")]
    pub message: String,
}

impl Success {
    pub fn new(success: bool, message: String) -> Success {
        Success {
            success: success,
            message: message,
        }
    }
}

/// Converts the Success value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for Success {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("success".to_string());
        params.push(self.success.to_string());

        params.push("message".to_string());
        params.push(self.message.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Success value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Success {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub success: Vec<bool>,
            pub message: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing Success".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "success" => intermediate_rep
                        .success
                        .push(bool::from_str(val).map_err(|x| format!("{}", x))?),
                    "message" => intermediate_rep
                        .message
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing Success".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Success {
            success: intermediate_rep
                .success
                .into_iter()
                .next()
                .ok_or("success missing in Success".to_string())?,
            message: intermediate_rep
                .message
                .into_iter()
                .next()
                .ok_or("message missing in Success".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebcaptureAutoBatch> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<WebcaptureAutoBatch>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<WebcaptureAutoBatch>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for WebcaptureAutoBatch - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<WebcaptureAutoBatch>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <WebcaptureAutoBatch as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into WebcaptureAutoBatch - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebcaptureAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::WebcaptureEntity>,
}

impl WebcaptureAutoBatch {
    pub fn new(
        editgroup: models::Editgroup,
        entity_list: Vec<models::WebcaptureEntity>,
    ) -> WebcaptureAutoBatch {
        WebcaptureAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

/// Converts the WebcaptureAutoBatch value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for WebcaptureAutoBatch {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editgroup in query parameter serialization

        // Skipping entity_list in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebcaptureAutoBatch value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebcaptureAutoBatch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup: Vec<models::Editgroup>,
            pub entity_list: Vec<Vec<models::WebcaptureEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing WebcaptureAutoBatch".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "entity_list" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in WebcaptureAutoBatch"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing WebcaptureAutoBatch".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebcaptureAutoBatch {
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in WebcaptureAutoBatch".to_string())?,
            entity_list: intermediate_rep
                .entity_list
                .into_iter()
                .next()
                .ok_or("entity_list missing in WebcaptureAutoBatch".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebcaptureCdxLine> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<WebcaptureCdxLine>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<WebcaptureCdxLine>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for WebcaptureCdxLine - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<WebcaptureCdxLine>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <WebcaptureCdxLine as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into WebcaptureCdxLine - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebcaptureCdxLine {
    /// \"Sortable URL\" format. See guide for details.
    #[serde(rename = "surt")]
    pub surt: String,

    /// Date and time of capture, in ISO format. UTC, 'Z'-terminated, second (or better) precision.
    #[serde(rename = "timestamp")]
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Full URL/URI of resource captured.
    #[serde(rename = "url")]
    pub url: String,

    /// Mimetype of the resource at this URL. May be the Content-Type header, or the actually sniffed file type.
    #[serde(rename = "mimetype")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mimetype: Option<String>,

    /// HTTP status code. Should generally be 200, especially for the primary resource, but may be 3xx (redirect) or even error codes if embedded resources can not be fetched successfully.
    #[serde(rename = "status_code")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i64>,

    /// Resource (file) size in bytes
    #[serde(rename = "size")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,

    /// SHA-1 hash of data, in hex encoding
    #[serde(rename = "sha1")]
    pub sha1: String,

    /// SHA-256 hash of data, in hex encoding
    #[serde(rename = "sha256")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

impl WebcaptureCdxLine {
    pub fn new(
        surt: String,
        timestamp: chrono::DateTime<chrono::Utc>,
        url: String,
        sha1: String,
    ) -> WebcaptureCdxLine {
        WebcaptureCdxLine {
            surt: surt,
            timestamp: timestamp,
            url: url,
            mimetype: None,
            status_code: None,
            size: None,
            sha1: sha1,
            sha256: None,
        }
    }
}

/// Converts the WebcaptureCdxLine value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for WebcaptureCdxLine {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("surt".to_string());
        params.push(self.surt.to_string());

        // Skipping timestamp in query parameter serialization

        params.push("url".to_string());
        params.push(self.url.to_string());

        if let Some(ref mimetype) = self.mimetype {
            params.push("mimetype".to_string());
            params.push(mimetype.to_string());
        }

        if let Some(ref status_code) = self.status_code {
            params.push("status_code".to_string());
            params.push(status_code.to_string());
        }

        if let Some(ref size) = self.size {
            params.push("size".to_string());
            params.push(size.to_string());
        }

        params.push("sha1".to_string());
        params.push(self.sha1.to_string());

        if let Some(ref sha256) = self.sha256 {
            params.push("sha256".to_string());
            params.push(sha256.to_string());
        }

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebcaptureCdxLine value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebcaptureCdxLine {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub surt: Vec<String>,
            pub timestamp: Vec<chrono::DateTime<chrono::Utc>>,
            pub url: Vec<String>,
            pub mimetype: Vec<String>,
            pub status_code: Vec<i64>,
            pub size: Vec<i64>,
            pub sha1: Vec<String>,
            pub sha256: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing WebcaptureCdxLine".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "surt" => intermediate_rep
                        .surt
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "timestamp" => intermediate_rep.timestamp.push(
                        chrono::DateTime::<chrono::Utc>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "url" => intermediate_rep
                        .url
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "mimetype" => intermediate_rep
                        .mimetype
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "status_code" => intermediate_rep
                        .status_code
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "size" => intermediate_rep
                        .size
                        .push(i64::from_str(val).map_err(|x| format!("{}", x))?),
                    "sha1" => intermediate_rep
                        .sha1
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "sha256" => intermediate_rep
                        .sha256
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing WebcaptureCdxLine".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebcaptureCdxLine {
            surt: intermediate_rep
                .surt
                .into_iter()
                .next()
                .ok_or("surt missing in WebcaptureCdxLine".to_string())?,
            timestamp: intermediate_rep
                .timestamp
                .into_iter()
                .next()
                .ok_or("timestamp missing in WebcaptureCdxLine".to_string())?,
            url: intermediate_rep
                .url
                .into_iter()
                .next()
                .ok_or("url missing in WebcaptureCdxLine".to_string())?,
            mimetype: intermediate_rep.mimetype.into_iter().next(),
            status_code: intermediate_rep.status_code.into_iter().next(),
            size: intermediate_rep.size.into_iter().next(),
            sha1: intermediate_rep
                .sha1
                .into_iter()
                .next()
                .ok_or("sha1 missing in WebcaptureCdxLine".to_string())?,
            sha256: intermediate_rep.sha256.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebcaptureEntity> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<WebcaptureEntity>>
    for hyper::header::HeaderValue
{
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<WebcaptureEntity>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for WebcaptureEntity - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue>
    for header::IntoHeaderValue<WebcaptureEntity>
{
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <WebcaptureEntity as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into WebcaptureEntity - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebcaptureEntity {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    #[serde(rename = "cdx")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdx: Option<Vec<models::WebcaptureCdxLine>>,

    #[serde(rename = "archive_urls")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_urls: Option<Vec<models::WebcaptureUrl>>,

    /// Base URL of the primary resource this is a capture of
    #[serde(rename = "original_url")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,

    /// Same format as CDX line timestamp (UTC, etc). Corresponds to the overall capture timestamp. Should generally be the timestamp of capture of the primary resource URL.
    #[serde(rename = "timestamp")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Set of identifier of release entities this fileset represents a full manifestation of. Usually a single release.
    #[serde(rename = "release_ids")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_ids: Option<Vec<String>>,

    /// Full release entities, included in GET responses when `releases` included in `expand` parameter. Ignored if included in PUT or POST requests.
    #[serde(rename = "releases")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases: Option<Vec<models::ReleaseEntity>>,
}

impl WebcaptureEntity {
    pub fn new() -> WebcaptureEntity {
        WebcaptureEntity {
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
            cdx: None,
            archive_urls: None,
            original_url: None,
            timestamp: None,
            release_ids: None,
            releases: None,
        }
    }
}

/// Converts the WebcaptureEntity value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for WebcaptureEntity {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref state) = self.state {
            params.push("state".to_string());
            params.push(state.to_string());
        }

        if let Some(ref ident) = self.ident {
            params.push("ident".to_string());
            params.push(ident.to_string());
        }

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref redirect) = self.redirect {
            params.push("redirect".to_string());
            params.push(redirect.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping edit_extra in query parameter serialization
        // Skipping edit_extra in query parameter serialization

        // Skipping cdx in query parameter serialization

        // Skipping archive_urls in query parameter serialization

        if let Some(ref original_url) = self.original_url {
            params.push("original_url".to_string());
            params.push(original_url.to_string());
        }

        // Skipping timestamp in query parameter serialization

        if let Some(ref release_ids) = self.release_ids {
            params.push("release_ids".to_string());
            params.push(
                release_ids
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
                    .to_string(),
            );
        }

        // Skipping releases in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebcaptureEntity value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebcaptureEntity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub redirect: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub edit_extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub cdx: Vec<Vec<models::WebcaptureCdxLine>>,
            pub archive_urls: Vec<Vec<models::WebcaptureUrl>>,
            pub original_url: Vec<String>,
            pub timestamp: Vec<chrono::DateTime<chrono::Utc>>,
            pub release_ids: Vec<Vec<String>>,
            pub releases: Vec<Vec<models::ReleaseEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing WebcaptureEntity".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep
                        .state
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect" => intermediate_rep
                        .redirect
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in WebcaptureEntity"
                            .to_string(),
                    ),
                    "edit_extra" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in WebcaptureEntity"
                            .to_string(),
                    ),
                    "cdx" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in WebcaptureEntity"
                            .to_string(),
                    ),
                    "archive_urls" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in WebcaptureEntity"
                            .to_string(),
                    ),
                    "original_url" => intermediate_rep
                        .original_url
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "timestamp" => intermediate_rep.timestamp.push(
                        chrono::DateTime::<chrono::Utc>::from_str(val)
                            .map_err(|x| format!("{}", x))?,
                    ),
                    "release_ids" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in WebcaptureEntity"
                            .to_string(),
                    ),
                    "releases" => return std::result::Result::Err(
                        "Parsing a container in this style is not supported in WebcaptureEntity"
                            .to_string(),
                    ),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing WebcaptureEntity".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebcaptureEntity {
            state: intermediate_rep.state.into_iter().next(),
            ident: intermediate_rep.ident.into_iter().next(),
            revision: intermediate_rep.revision.into_iter().next(),
            redirect: intermediate_rep.redirect.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            edit_extra: intermediate_rep.edit_extra.into_iter().next(),
            cdx: intermediate_rep.cdx.into_iter().next(),
            archive_urls: intermediate_rep.archive_urls.into_iter().next(),
            original_url: intermediate_rep.original_url.into_iter().next(),
            timestamp: intermediate_rep.timestamp.into_iter().next(),
            release_ids: intermediate_rep.release_ids.into_iter().next(),
            releases: intermediate_rep.releases.into_iter().next(),
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WebcaptureUrl> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<WebcaptureUrl>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<WebcaptureUrl>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for WebcaptureUrl - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<WebcaptureUrl> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <WebcaptureUrl as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into WebcaptureUrl - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WebcaptureUrl {
    /// URL/URI pointing to archive of this web resource.
    #[serde(rename = "url")]
    pub url: String,

    /// Type of archive endpoint. Usually `wayback` (WBM replay of primary resource), or `warc` (direct URL to a WARC file containing all resources of the capture). See guide for full list.
    #[serde(rename = "rel")]
    pub rel: String,
}

impl WebcaptureUrl {
    pub fn new(url: String, rel: String) -> WebcaptureUrl {
        WebcaptureUrl { url: url, rel: rel }
    }
}

/// Converts the WebcaptureUrl value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for WebcaptureUrl {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        params.push("url".to_string());
        params.push(self.url.to_string());

        params.push("rel".to_string());
        params.push(self.rel.to_string());

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WebcaptureUrl value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WebcaptureUrl {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub url: Vec<String>,
            pub rel: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing WebcaptureUrl".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "url" => intermediate_rep
                        .url
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "rel" => intermediate_rep
                        .rel
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing WebcaptureUrl".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WebcaptureUrl {
            url: intermediate_rep
                .url
                .into_iter()
                .next()
                .ok_or("url missing in WebcaptureUrl".to_string())?,
            rel: intermediate_rep
                .rel
                .into_iter()
                .next()
                .ok_or("rel missing in WebcaptureUrl".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WorkAutoBatch> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<WorkAutoBatch>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<WorkAutoBatch>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for WorkAutoBatch - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<WorkAutoBatch> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <WorkAutoBatch as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into WorkAutoBatch - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WorkAutoBatch {
    #[serde(rename = "editgroup")]
    pub editgroup: models::Editgroup,

    #[serde(rename = "entity_list")]
    pub entity_list: Vec<models::WorkEntity>,
}

impl WorkAutoBatch {
    pub fn new(
        editgroup: models::Editgroup,
        entity_list: Vec<models::WorkEntity>,
    ) -> WorkAutoBatch {
        WorkAutoBatch {
            editgroup: editgroup,
            entity_list: entity_list,
        }
    }
}

/// Converts the WorkAutoBatch value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for WorkAutoBatch {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];
        // Skipping editgroup in query parameter serialization

        // Skipping entity_list in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WorkAutoBatch value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WorkAutoBatch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub editgroup: Vec<models::Editgroup>,
            pub entity_list: Vec<Vec<models::WorkEntity>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing WorkAutoBatch".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "editgroup" => intermediate_rep
                        .editgroup
                        .push(models::Editgroup::from_str(val).map_err(|x| format!("{}", x))?),
                    "entity_list" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in WorkAutoBatch"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing WorkAutoBatch".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WorkAutoBatch {
            editgroup: intermediate_rep
                .editgroup
                .into_iter()
                .next()
                .ok_or("editgroup missing in WorkAutoBatch".to_string())?,
            entity_list: intermediate_rep
                .entity_list
                .into_iter()
                .next()
                .ok_or("entity_list missing in WorkAutoBatch".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<WorkEntity> and hyper::header::HeaderValue

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<header::IntoHeaderValue<WorkEntity>> for hyper::header::HeaderValue {
    type Error = String;

    fn try_from(
        hdr_value: header::IntoHeaderValue<WorkEntity>,
    ) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match hyper::header::HeaderValue::from_str(&hdr_value) {
            std::result::Result::Ok(value) => std::result::Result::Ok(value),
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Invalid header value for WorkEntity - value: {} is invalid {}",
                hdr_value, e
            )),
        }
    }
}

#[cfg(any(feature = "client", feature = "server"))]
impl std::convert::TryFrom<hyper::header::HeaderValue> for header::IntoHeaderValue<WorkEntity> {
    type Error = String;

    fn try_from(hdr_value: hyper::header::HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
            std::result::Result::Ok(value) => {
                match <WorkEntity as std::str::FromStr>::from_str(value) {
                    std::result::Result::Ok(value) => {
                        std::result::Result::Ok(header::IntoHeaderValue(value))
                    }
                    std::result::Result::Err(err) => std::result::Result::Err(format!(
                        "Unable to convert header value '{}' into WorkEntity - {}",
                        value, err
                    )),
                }
            }
            std::result::Result::Err(e) => std::result::Result::Err(format!(
                "Unable to convert header: {:?} to string: {}",
                hdr_value, e
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct WorkEntity {
    // Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "state")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "ident")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ident: Option<String>,

    /// UUID (lower-case, dash-separated, hex-encoded 128-bit)
    #[serde(rename = "revision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// base32-encoded unique identifier
    #[serde(rename = "redirect")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.
    #[serde(rename = "extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).
    #[serde(rename = "edit_extra")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_extra: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl WorkEntity {
    pub fn new() -> WorkEntity {
        WorkEntity {
            state: None,
            ident: None,
            revision: None,
            redirect: None,
            extra: None,
            edit_extra: None,
        }
    }
}

/// Converts the WorkEntity value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::string::ToString for WorkEntity {
    fn to_string(&self) -> String {
        let mut params: Vec<String> = vec![];

        if let Some(ref state) = self.state {
            params.push("state".to_string());
            params.push(state.to_string());
        }

        if let Some(ref ident) = self.ident {
            params.push("ident".to_string());
            params.push(ident.to_string());
        }

        if let Some(ref revision) = self.revision {
            params.push("revision".to_string());
            params.push(revision.to_string());
        }

        if let Some(ref redirect) = self.redirect {
            params.push("redirect".to_string());
            params.push(redirect.to_string());
        }

        // Skipping extra in query parameter serialization
        // Skipping extra in query parameter serialization

        // Skipping edit_extra in query parameter serialization
        // Skipping edit_extra in query parameter serialization

        params.join(",").to_string()
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a WorkEntity value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for WorkEntity {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        #[derive(Default)]
        // An intermediate representation of the struct to use for parsing.
        struct IntermediateRep {
            pub state: Vec<String>,
            pub ident: Vec<String>,
            pub revision: Vec<String>,
            pub redirect: Vec<String>,
            pub extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
            pub edit_extra: Vec<std::collections::HashMap<String, serde_json::Value>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',').into_iter();
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => {
                    return std::result::Result::Err(
                        "Missing value while parsing WorkEntity".to_string(),
                    )
                }
            };

            if let Some(key) = key_result {
                match key {
                    "state" => intermediate_rep
                        .state
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "ident" => intermediate_rep
                        .ident
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "revision" => intermediate_rep
                        .revision
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "redirect" => intermediate_rep
                        .redirect
                        .push(String::from_str(val).map_err(|x| format!("{}", x))?),
                    "extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in WorkEntity"
                                .to_string(),
                        )
                    }
                    "edit_extra" => {
                        return std::result::Result::Err(
                            "Parsing a container in this style is not supported in WorkEntity"
                                .to_string(),
                        )
                    }
                    _ => {
                        return std::result::Result::Err(
                            "Unexpected key while parsing WorkEntity".to_string(),
                        )
                    }
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(WorkEntity {
            state: intermediate_rep.state.into_iter().next(),
            ident: intermediate_rep.ident.into_iter().next(),
            revision: intermediate_rep.revision.into_iter().next(),
            redirect: intermediate_rep.redirect.into_iter().next(),
            extra: intermediate_rep.extra.into_iter().next(),
            edit_extra: intermediate_rep.edit_extra.into_iter().next(),
        })
    }
}
