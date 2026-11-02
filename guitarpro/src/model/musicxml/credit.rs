use serde::{Deserialize, Serialize};

/// A text block placed on a page of the score (title, subtitle, composer, etc.).
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditWords {
    #[serde(rename = "@justify", skip_serializing_if = "Option::is_none")]
    pub justify: Option<String>,
    #[serde(rename = "@valign", skip_serializing_if = "Option::is_none")]
    pub valign: Option<String>,
    #[serde(rename = "@font-size", skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(rename = "@font-weight", skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// An image placed on a page of the score.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditImage {
    #[serde(rename = "@source")]
    pub source: String,
    #[serde(rename = "@type")]
    pub image_type: String,
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "@halign", skip_serializing_if = "Option::is_none")]
    pub halign: Option<String>,
    #[serde(rename = "@valign", skip_serializing_if = "Option::is_none")]
    pub valign: Option<String>,
}

/// A SMuFL symbol used as a credit element.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditSymbol {
    #[serde(rename = "@default-x", skip_serializing_if = "Option::is_none")]
    pub default_x: Option<f64>,
    #[serde(rename = "@default-y", skip_serializing_if = "Option::is_none")]
    pub default_y: Option<f64>,
    #[serde(rename = "$text")]
    pub value: String,
}

/// One content item inside a `<credit>` block.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CreditContent {
    CreditType(String),
    Link(CreditLink),
    Bookmark(CreditBookmark),
    CreditImage(CreditImage),
    CreditWords(CreditWords),
    CreditSymbol(CreditSymbol),
}

/// A hyperlink inside a credit block.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditLink {
    #[serde(rename = "@xlink:href", skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(rename = "@xlink:type", skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A named bookmark inside a credit block.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreditBookmark {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A page-level text, image, or symbol element (title, composer name, copyright, etc.).
///
/// Maps to the `<credit>` element.
#[derive(Debug, Serialize, Deserialize)]
pub struct Credit {
    /// Which page this credit appears on (1-based). Defaults to 1 if absent.
    #[serde(rename = "@page", skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(rename = "$value", default)]
    pub content: Vec<CreditContent>,
}
