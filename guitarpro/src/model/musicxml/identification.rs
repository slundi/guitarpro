use serde::Deserialize;

/// A person or organization who contributed to the score.
///
/// The `type` attribute specifies the role, e.g. `"composer"`, `"lyricist"`, `"arranger"`.
#[derive(Debug, Deserialize)]
pub struct Creator {
    #[serde(rename = "@type")]
    pub creator_type: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// A rights statement for the score.
#[derive(Debug, Deserialize)]
pub struct Rights {
    #[serde(rename = "@type")]
    pub rights_type: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Records software and date information about how the file was encoded.
#[derive(Debug, Deserialize)]
pub struct Encoding {
    #[serde(rename = "encoding-date")]
    pub encoding_date: Option<String>,
    #[serde(rename = "encoder", default)]
    pub encoders: Vec<TypedText>,
    #[serde(rename = "software", default)]
    pub software: Vec<String>,
    #[serde(rename = "encoding-description", default)]
    pub encoding_description: Vec<String>,
    #[serde(rename = "supports", default)]
    pub supports: Vec<Supports>,
}

/// A text element that carries an optional `type` attribute.
#[derive(Debug, Deserialize)]
pub struct TypedText {
    #[serde(rename = "@type")]
    pub text_type: Option<String>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Indicates whether the encoding supports a specific MusicXML element or attribute.
///
/// Used in `<encoding>` to declare what features the encoder actively uses.
#[derive(Debug, Deserialize)]
pub struct Supports {
    /// `"yes"` or `"no"`.
    #[serde(rename = "@type")]
    pub supports_type: String,
    #[serde(rename = "@element")]
    pub element: String,
    #[serde(rename = "@attribute")]
    pub attribute: Option<String>,
    #[serde(rename = "@value")]
    pub value: Option<String>,
}

/// A key/value pair inside `<miscellaneous>`.
#[derive(Debug, Deserialize)]
pub struct MiscellaneousField {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub value: String,
}

/// Free-form metadata fields that do not fit other identification categories.
#[derive(Debug, Deserialize)]
pub struct Miscellaneous {
    #[serde(rename = "miscellaneous-field", default)]
    pub fields: Vec<MiscellaneousField>,
}

/// Identification information for the score: authorship, rights, encoding history.
///
/// Maps to the `<identification>` element.
#[derive(Debug, Deserialize)]
pub struct Identification {
    #[serde(rename = "creator", default)]
    pub creators: Vec<Creator>,
    #[serde(rename = "rights", default)]
    pub rights: Vec<Rights>,
    pub encoding: Option<Encoding>,
    pub source: Option<String>,
    #[serde(rename = "relation", default)]
    pub relations: Vec<TypedText>,
    pub miscellaneous: Option<Miscellaneous>,
}
