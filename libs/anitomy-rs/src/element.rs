use std::borrow::Cow;

use crate::tokenizer::Token;

/// The kind of element that has been parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ElementKind {
    AudioTerm,
    DeviceCompatibility,
    Episode,
    EpisodeTitle,
    EpisodeAlt,
    FileChecksum,
    FileExtension,
    Language,
    Other,
    ReleaseGroup,
    ReleaseInformation,
    ReleaseVersion,
    Season,
    Source,
    Subtitles,
    Title,
    Type,
    VideoResolution,
    VideoTerm,
    Volume,
    Year,
    Date,
}

impl ElementKind {
    /// Returns this ElementKind as a human representable string
    pub const fn as_str(&self) -> &'static str {
        match self {
            ElementKind::AudioTerm => "Audio Term",
            ElementKind::DeviceCompatibility => "Device Compatibility",
            ElementKind::Episode => "Episode",
            ElementKind::EpisodeTitle => "Episode Title",
            ElementKind::EpisodeAlt => "Episode Alt",
            ElementKind::FileChecksum => "File Checksum",
            ElementKind::FileExtension => "File Extension",
            ElementKind::Language => "Language",
            ElementKind::Other => "Other",
            ElementKind::ReleaseGroup => "Release Group",
            ElementKind::ReleaseInformation => "Release Information",
            ElementKind::ReleaseVersion => "Release Version",
            ElementKind::Season => "Season",
            ElementKind::Source => "Source",
            ElementKind::Subtitles => "Subtitles",
            ElementKind::Title => "Title",
            ElementKind::Type => "Type",
            ElementKind::VideoResolution => "Video Resolution",
            ElementKind::VideoTerm => "Video Term",
            ElementKind::Volume => "Volume",
            ElementKind::Year => "Year",
            ElementKind::Date => "Date",
        }
    }
}

/// A parsed element
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Element<'a> {
    #[cfg_attr(feature = "serde", serde(rename = "type"))]
    pub(crate) kind: ElementKind,
    #[cfg_attr(feature = "serde", serde(borrow))]
    pub(crate) value: Cow<'a, str>,
    #[cfg_attr(feature = "serde", serde(default, skip))]
    pub(crate) position: usize,
}

impl<'a> Element<'a> {
    pub(crate) fn new(kind: ElementKind, token: &Token<'a>) -> Self {
        Self {
            kind,
            value: token.value.into(),
            position: token.position,
        }
    }

    /// Returns the kind of element
    pub fn kind(&self) -> ElementKind {
        self.kind
    }

    /// Returns the value of the element
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// A helper type that turns a slice of [`Element`] objects into a flat struct with multiple elements.
///
/// This makes it easier to work with when, e.g. encoding it into a JSON data structure or something
/// similar. Note that `Option` keys that are `None` are not serialized.
///
/// If multiple elements are found with the same [`ElementKind`], then the latest one is stored.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElementObject<'a> {
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub audio_term: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub device_compatibility: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub episode: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub episode_alt: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub episode_title: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub file_checksum: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub file_extension: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub language: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub other: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub release_group: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub release_information: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub release_version: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub season: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub source: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub subtitles: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(
            rename = "type",
            borrow,
            default,
            skip_serializing_if = "Option::is_none"
        )
    )]
    pub kind: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub video_resolution: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub video_term: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub volume: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub year: Option<Cow<'a, str>>,
    #[cfg_attr(
        feature = "serde",
        serde(borrow, default, skip_serializing_if = "Option::is_none")
    )]
    pub date: Option<Cow<'a, str>>,
}

/// A helper type that turns a slice of [`Element`] objects into a flat struct with multiple elements.
///
/// This makes it easier to work with when, e.g. encoding it into a JSON data structure or something
/// similar. Note that `Option` keys that are `None` are not serialized.
///
/// If multiple elements are found with the same [`ElementKind`], then the latest one is stored.
///
/// Unlike [`ElementObject`], this one only contains owned strings which makes it easier to work
/// with for the deserialization use case.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OwnedElementObject {
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub audio_term: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub device_compatibility: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub episode: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub episode_alt: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub episode_title: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub file_checksum: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub file_extension: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub language: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub other: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub release_group: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub release_information: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub release_version: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub season: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub source: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub subtitles: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub title: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(rename = "type", default, skip_serializing_if = "Option::is_none")
    )]
    pub kind: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub video_resolution: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub video_term: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub volume: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub year: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub date: Option<String>,
}

macro_rules! impl_from_iterator {
    ($($name:ident => $mapped:ident),+$(,)?) => {
        impl<'a, 'b: 'a> FromIterator<&'b Element<'a>> for ElementObject<'a> {
            fn from_iter<T: IntoIterator<Item = &'b Element<'a>>>(iter: T) -> Self {
                use std::borrow::Borrow;
                let mut object = Self::default();
                for element in iter {
                    match element.kind {
                        $(
                            $crate::ElementKind::$name => object.$mapped = Some(std::borrow::Cow::Borrowed(element.value.borrow()))
                        ),+
                    }
                }
                object
            }
        }

        impl<'a> FromIterator<Element<'a>> for ElementObject<'a> {
            fn from_iter<T: IntoIterator<Item = Element<'a>>>(iter: T) -> Self {
                let mut object = Self::default();
                for element in iter {
                    match element.kind {
                        $(
                            $crate::ElementKind::$name => object.$mapped = Some(element.value)
                        ),+
                    }
                }
                object
            }
        }

        impl<'a, 'b: 'a> FromIterator<&'b Element<'a>> for OwnedElementObject {
            fn from_iter<T: IntoIterator<Item = &'b Element<'a>>>(iter: T) -> Self {
                let mut object = Self::default();
                for element in iter {
                    match element.kind {
                        $(
                            $crate::ElementKind::$name => object.$mapped = Some(String::from(&element.value[..]))
                        ),+
                    }
                }
                object
            }
        }

        impl<'a> FromIterator<Element<'a>> for OwnedElementObject {
            fn from_iter<T: IntoIterator<Item = Element<'a>>>(iter: T) -> Self {
                let mut object = Self::default();
                for element in iter {
                    match element.kind {
                        $(
                            $crate::ElementKind::$name => object.$mapped = Some(element.value.into_owned())
                        ),+
                    }
                }
                object
            }
        }
    };
}

impl_from_iterator! {
    AudioTerm => audio_term,
    DeviceCompatibility => device_compatibility,
    Episode => episode,
    EpisodeAlt => episode_alt,
    EpisodeTitle => episode_title,
    FileChecksum => file_checksum,
    FileExtension => file_extension,
    Language => language,
    Other => other,
    ReleaseGroup => release_group,
    ReleaseInformation => release_information,
    ReleaseVersion => release_version,
    Season => season,
    Source => source,
    Subtitles => subtitles,
    Title => title,
    Type => kind,
    VideoResolution => video_resolution,
    VideoTerm => video_term,
    Volume => volume,
    Year => year,
    Date => date,
}
