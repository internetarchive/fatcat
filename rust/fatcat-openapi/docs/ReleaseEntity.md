# ReleaseEntity

## Properties
Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**state** | **String** |  | [optional] [default to None]
**ident** | **String** | base32-encoded unique identifier | [optional] [default to None]
**revision** | **String** | UUID (lower-case, dash-separated, hex-encoded 128-bit) | [optional] [default to None]
**redirect** | **String** | base32-encoded unique identifier | [optional] [default to None]
**extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with the other entity metadata. See guide for (unenforced) schema conventions.  | [optional] [default to None]
**edit_extra** | [**std::collections::HashMap<String, serde_json::Value>**](object.md) | Free-form JSON metadata that will be stored with specific entity edits (eg, creation/update/delete).  | [optional] [default to None]
**title** | **String** | Required for valid entities. The title used in citations and for display. Sometimes the English translation of title e even if release content is not English.  | [optional] [default to None]
**subtitle** | **String** | Subtitle of release. In many cases, better to merge with title than include as separate field (unless combined title would be very long). See guide for details.  | [optional] [default to None]
**original_title** | **String** | Title in original language if `title` field has been translated. See guide for details.  | [optional] [default to None]
**work_id** | **String** | Identifier of work this release is part of. In creation (POST) requests, a work entity will be created automatically if this field is not set.  | [optional] [default to None]
**container** | [***models::ContainerEntity**](container_entity.md) |  | [optional] [default to None]
**files** | [**Vec<models::FileEntity>**](file_entity.md) | Complete file entities identified by `file_ids` field. Only included in GET responses when `files` included in `expand` parameter; ignored in PUT or POST requests.  | [optional] [default to None]
**filesets** | [**Vec<models::FilesetEntity>**](fileset_entity.md) | Complete file entities identified by `filesets_ids` field. Only included in GET responses when `filesets` included in `expand` parameter; ignored in PUT or POST requests.  | [optional] [default to None]
**webcaptures** | [**Vec<models::WebcaptureEntity>**](webcapture_entity.md) | Complete webcapture entities identified by `webcapture_ids` field. Only included in GET responses when `webcaptures` included in `expand` parameter; ignored in PUT or POST requests.  | [optional] [default to None]
**container_id** | **String** | Used to link this release to a container entity that the release was published as part of.  | [optional] [default to None]
**release_type** | **String** | \"Type\" or \"medium\" that this release is published as. See guide for valid values.  | [optional] [default to None]
**release_stage** | **String** | The stage of publication of this specific release. See guide for valid values and semantics.  | [optional] [default to None]
**release_date** | [***chrono::DateTime::<chrono::Utc>**](date.md) | Full date when this release was formally published. ISO format, like `2019-03-05`. See guide for semantics.  | [optional] [default to None]
**release_year** | **i64** | Year when this release was formally published. Must match `release_date` if that field is set; this field exists because sometimes only the year is known.  | [optional] [default to None]
**withdrawn_status** | **String** | Type of withdrawl or retraction of this release, if applicable. If release has not been withdrawn, should be `null` (aka, not set, not the string \"null\" or an empty string).  | [optional] [default to None]
**withdrawn_date** | [***chrono::DateTime::<chrono::Utc>**](date.md) | Full date when this release was formally withdrawn (if applicable). ISO format, like `release_date`.  | [optional] [default to None]
**withdrawn_year** | **i64** | Year corresponding with `withdrawn_date` like `release_year`/`release_date`.  | [optional] [default to None]
**ext_ids** | [***models::ReleaseExtIds**](release_ext_ids.md) |  | 
**volume** | **String** | Volume number of container that this release was published in. Often corresponds to the \"Nth\" year of publication, but can be any string. See guide.  | [optional] [default to None]
**issue** | **String** | Issue number of volume/container that this release was published in. Sometimes coresponds to a month number in the year, but can be any string. See guide.  | [optional] [default to None]
**pages** | **String** | Either a single page number (\"first page\") or a range of pages separated by a dash (\"-\"). See guide for details.  | [optional] [default to None]
**number** | **String** | For, eg, technical reports, which are published in series or assigned some other institutional or container-specific identifier.  | [optional] [default to None]
**version** | **String** | For, eg, updated technical reports or software packages, where the version string may be the only field disambiguating between releases.  | [optional] [default to None]
**publisher** | **String** | Name, usually English, of the entity or institution responsible for publication of this release. Not necessarily the imprint/brand. See guide.  | [optional] [default to None]
**language** | **String** | Primary language of the content of the full release. Two-letter RFC1766/ISO639-1 language code, with some custom extensions/additions. See guide.  | [optional] [default to None]
**license_slug** | **String** | Short string (slug) name of license under which release is openly published (if applicable).  | [optional] [default to None]
**contribs** | [**Vec<models::ReleaseContrib>**](release_contrib.md) |  | [optional] [default to None]
**refs** | [**Vec<models::ReleaseRef>**](release_ref.md) |  | [optional] [default to None]
**abstracts** | [**Vec<models::ReleaseAbstract>**](release_abstract.md) |  | [optional] [default to None]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


