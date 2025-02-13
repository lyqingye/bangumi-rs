use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum KeywordKind {
    AudioChannels,
    AudioCodec,
    AudioLanguage,
    DeviceCompatibility,
    Episode,
    EpisodeType,
    FileExtension,
    Language,
    Other,
    ReleaseGroup,
    ReleaseInformation,
    ReleaseVersion,
    Season,
    Source,
    Subtitles,
    Type,
    VideoCodec,
    VideoColorDepth,
    VideoFormat,
    VideoFrameRate,
    VideoProfile,
    VideoQuality,
    VideoResolution,
    Volume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Keyword {
    pub(crate) kind: KeywordKind,
    flags: u8,
}

impl Keyword {
    const AMBIGUOUS: u8 = 1 << 0;
    const UNBOUNDED: u8 = 1 << 1;

    pub(crate) const fn new(kind: KeywordKind) -> Self {
        Self { kind, flags: 0 }
    }

    pub(crate) const fn unbounded(kind: KeywordKind) -> Self {
        Self {
            kind,
            flags: Self::UNBOUNDED,
        }
    }

    pub(crate) const fn ambiguous(kind: KeywordKind) -> Self {
        Self {
            kind,
            flags: Self::AMBIGUOUS,
        }
    }

    pub(crate) const fn is_ambiguous(&self) -> bool {
        (self.flags & Self::AMBIGUOUS) == Self::AMBIGUOUS
    }

    pub(crate) const fn is_bounded(&self) -> bool {
        (self.flags & Self::UNBOUNDED) != Self::UNBOUNDED
    }
}

#[derive(Debug, Clone)]
pub struct KeywordConfig {
    keywords: HashMap<String, Keyword>,
}

impl Default for KeywordConfig {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl KeywordConfig {
    pub fn new() -> Self {
        Self {
            keywords: HashMap::new(),
        }
    }

    pub(crate) fn with_defaults() -> Self {
        let mut config = Self::new();
        // Audio
        //
        // Channels
        config.add_keyword("2.0ch", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("2ch", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("5.1", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("5.1ch", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("7.1", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("7.1ch", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("DTS", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("DTS-ES", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("DTS5.1", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("Dolby TrueHD", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("TrueHD", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("TrueHD5.1", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("DD5.1", Keyword::new(KeywordKind::AudioChannels));
        config.add_keyword("DD2.0", Keyword::new(KeywordKind::AudioChannels));
        // Codec
        config.add_keyword("AAC", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("AAC2.0", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("AACX2", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("AACX3", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("AACX4", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("AC3", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("EAC3", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("E-AC-3", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("FLAC", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("FLACX2", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("FLACX3", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("FLACX4", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("Lossless", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("MP3", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("OGG", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("Vorbis", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("Atmos", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("Dolby Atmos", Keyword::new(KeywordKind::AudioCodec));
        config.add_keyword("Opus", Keyword::ambiguous(KeywordKind::AudioCodec)); // e.g. "Opus.COLORs"
                                                                                 // Language
        config.add_keyword("DualAudio", Keyword::new(KeywordKind::AudioLanguage));
        config.add_keyword("Dual Audio", Keyword::new(KeywordKind::AudioLanguage));
        config.add_keyword("Dual-Audio", Keyword::new(KeywordKind::AudioLanguage));

        // Device compatibility
        config.add_keyword(
            "Android",
            Keyword::ambiguous(KeywordKind::DeviceCompatibility),
        ); // e.g. "Dragon Ball Z: Super Android 13"
        config.add_keyword("iPad3", Keyword::new(KeywordKind::DeviceCompatibility));
        config.add_keyword("iPhone5", Keyword::new(KeywordKind::DeviceCompatibility));
        config.add_keyword("iPod", Keyword::new(KeywordKind::DeviceCompatibility));
        config.add_keyword("PS3", Keyword::new(KeywordKind::DeviceCompatibility));
        config.add_keyword("Xbox", Keyword::new(KeywordKind::DeviceCompatibility));
        config.add_keyword("Xbox360", Keyword::new(KeywordKind::DeviceCompatibility));

        // Episode prefix
        config.add_keyword("Ep", Keyword::new(KeywordKind::Episode));
        config.add_keyword("Eps", Keyword::new(KeywordKind::Episode));
        config.add_keyword("Episode", Keyword::new(KeywordKind::Episode));
        config.add_keyword("Episodes", Keyword::new(KeywordKind::Episode));
        config.add_keyword("Episodio", Keyword::new(KeywordKind::Episode));
        config.add_keyword("Episódio", Keyword::new(KeywordKind::Episode));
        config.add_keyword("Capitulo", Keyword::new(KeywordKind::Episode));
        config.add_keyword("Folge", Keyword::new(KeywordKind::Episode));

        // Episode type
        config.add_keyword("OP", Keyword::ambiguous(KeywordKind::EpisodeType));
        config.add_keyword("Opening", Keyword::ambiguous(KeywordKind::EpisodeType));
        config.add_keyword("ED", Keyword::ambiguous(KeywordKind::EpisodeType));
        config.add_keyword("Ending", Keyword::ambiguous(KeywordKind::EpisodeType));
        config.add_keyword("NCED", Keyword::new(KeywordKind::EpisodeType));
        config.add_keyword("NCOP", Keyword::new(KeywordKind::EpisodeType));
        config.add_keyword("Preview", Keyword::ambiguous(KeywordKind::EpisodeType));
        config.add_keyword("PV", Keyword::ambiguous(KeywordKind::EpisodeType));

        // File extension
        config.add_keyword("3gp", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("avi", Keyword::new(KeywordKind::FileExtension));
        // UncasedStr::new("divx")         =>    Keyword::new(KeywordKind::FileExtension),
        config.add_keyword("flv", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("m2ts", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("mkv", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("mov", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("mp4", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("mpg", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("ogm", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("rm", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("rmvb", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("ts", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("webm", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("wmv", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("ass", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("srt", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("ssa", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("7z", Keyword::new(KeywordKind::FileExtension));
        config.add_keyword("zip", Keyword::new(KeywordKind::FileExtension));

        // Language
        config.add_keyword("ENG", Keyword::new(KeywordKind::Language));
        config.add_keyword("English", Keyword::new(KeywordKind::Language));
        config.add_keyword("ESP", Keyword::ambiguous(KeywordKind::Language)); // e.g. "Tokyo ESP"
        config.add_keyword("Espanol", Keyword::new(KeywordKind::Language));
        config.add_keyword("Spanish", Keyword::new(KeywordKind::Language));
        config.add_keyword("ITA", Keyword::ambiguous(KeywordKind::Language)); // e.g. "Bokura ga Ita"
        config.add_keyword("JAP", Keyword::new(KeywordKind::Language));
        config.add_keyword("JP", Keyword::new(KeywordKind::Language));
        config.add_keyword("JA", Keyword::new(KeywordKind::Language));
        config.add_keyword("JPN", Keyword::new(KeywordKind::Language));
        config.add_keyword("PT-BR", Keyword::new(KeywordKind::Language));
        config.add_keyword("VOSTFR", Keyword::new(KeywordKind::Language));
        config.add_keyword("CHT", Keyword::new(KeywordKind::Language));
        config.add_keyword("CHS", Keyword::new(KeywordKind::Language));
        config.add_keyword("CHI", Keyword::new(KeywordKind::Language));
        config.add_keyword("简繁", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("简体", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("简中", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("繁体", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("简日", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("繁日", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("简英", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("BIG5", Keyword::unbounded(KeywordKind::Language));
        config.add_keyword("GB", Keyword::unbounded(KeywordKind::Language));

        // Other
        config.add_keyword("Remaster", Keyword::new(KeywordKind::Other));
        config.add_keyword("Remastered", Keyword::new(KeywordKind::Other));
        config.add_keyword("Uncensored", Keyword::new(KeywordKind::Other));
        config.add_keyword("Uncut", Keyword::new(KeywordKind::Other));
        // UncasedStr::new("TS")           =>    Keyword::new(KeywordKind::Other),
        config.add_keyword("VFR", Keyword::new(KeywordKind::Other));
        config.add_keyword("Widescreen", Keyword::new(KeywordKind::Other));
        config.add_keyword("WS", Keyword::new(KeywordKind::Other));

        // Release group
        config.add_keyword("THORA", Keyword::new(KeywordKind::ReleaseGroup)); // special case because usually placed at the end
        config.add_keyword("UTW-THORA", Keyword::new(KeywordKind::ReleaseGroup)); // due to special case above, parser can't handle compound ones
        config.add_keyword("JPTVclub", Keyword::new(KeywordKind::ReleaseGroup)); // usually at the end

        // Release information
        config.add_keyword("Batch", Keyword::new(KeywordKind::ReleaseInformation));
        config.add_keyword("Complete", Keyword::new(KeywordKind::ReleaseInformation));
        config.add_keyword("End", Keyword::ambiguous(KeywordKind::ReleaseInformation)); // e.g. "The End of Evangelion"
        config.add_keyword("Final", Keyword::ambiguous(KeywordKind::ReleaseInformation)); // e.g. "Final Approach"
        config.add_keyword("Patch", Keyword::new(KeywordKind::ReleaseInformation));
        config.add_keyword("Remux", Keyword::new(KeywordKind::ReleaseInformation));

        // Release version
        config.add_keyword("v0", Keyword::new(KeywordKind::ReleaseVersion));
        config.add_keyword("v1", Keyword::new(KeywordKind::ReleaseVersion));
        config.add_keyword("v2", Keyword::new(KeywordKind::ReleaseVersion));
        config.add_keyword("v3", Keyword::new(KeywordKind::ReleaseVersion));
        config.add_keyword("v4", Keyword::new(KeywordKind::ReleaseVersion));

        // Season
        // Usually preceded or followed by a number (e.g. `2nd Season` or `Season 2`).
        config.add_keyword("Season", Keyword::ambiguous(KeywordKind::Season));
        config.add_keyword("Saison", Keyword::ambiguous(KeywordKind::Season));

        // Source
        //
        // Blu-ray
        config.add_keyword("BD", Keyword::new(KeywordKind::Source));
        config.add_keyword("BDRip", Keyword::new(KeywordKind::Source));
        config.add_keyword("BluRay", Keyword::new(KeywordKind::Source));
        config.add_keyword("Blu-ray", Keyword::new(KeywordKind::Source));
        // DVD
        config.add_keyword("DVD", Keyword::new(KeywordKind::Source));
        config.add_keyword("DVD5", Keyword::new(KeywordKind::Source));
        config.add_keyword("DVD9", Keyword::new(KeywordKind::Source));
        config.add_keyword("DVDISO", Keyword::new(KeywordKind::Source));
        config.add_keyword("DVDRip", Keyword::new(KeywordKind::Source));
        config.add_keyword("DVD-Rip", Keyword::new(KeywordKind::Source));
        config.add_keyword("R2DVD", Keyword::new(KeywordKind::Source));
        config.add_keyword("R2J", Keyword::new(KeywordKind::Source));
        config.add_keyword("R2JDVD", Keyword::new(KeywordKind::Source));
        config.add_keyword("R2JDVDRip", Keyword::new(KeywordKind::Source));
        // TV
        config.add_keyword("HDTV", Keyword::new(KeywordKind::Source));
        config.add_keyword("HDTVRip", Keyword::new(KeywordKind::Source));
        config.add_keyword("TVRip", Keyword::new(KeywordKind::Source));
        config.add_keyword("TV-Rip", Keyword::new(KeywordKind::Source));
        // Web
        config.add_keyword("Web", Keyword::ambiguous(KeywordKind::Source));
        config.add_keyword("Webcast", Keyword::new(KeywordKind::Source));
        config.add_keyword("WebDL", Keyword::new(KeywordKind::Source));
        config.add_keyword("Web-DL", Keyword::new(KeywordKind::Source));
        config.add_keyword("WebRip", Keyword::new(KeywordKind::Source));
        config.add_keyword("AMZN", Keyword::new(KeywordKind::Source)); // Amazon Prime
        config.add_keyword("CR", Keyword::new(KeywordKind::Source)); // Crunchyroll
        config.add_keyword("Crunchyroll", Keyword::new(KeywordKind::Source));
        config.add_keyword("DSNP", Keyword::new(KeywordKind::Source)); // Disney+
        config.add_keyword("Funi", Keyword::new(KeywordKind::Source)); // Funimation
        config.add_keyword("Funimation", Keyword::new(KeywordKind::Source));
        config.add_keyword("HIDI", Keyword::new(KeywordKind::Source)); // Hidive
        config.add_keyword("Hidive", Keyword::new(KeywordKind::Source));
        config.add_keyword("Hulu", Keyword::new(KeywordKind::Source));
        config.add_keyword("Netflix", Keyword::new(KeywordKind::Source));
        config.add_keyword("NF", Keyword::new(KeywordKind::Source)); // Netflix
        config.add_keyword("VRV", Keyword::new(KeywordKind::Source));
        config.add_keyword("YouTube", Keyword::new(KeywordKind::Source));

        // Subtitles
        // UncasedStr::new("ASS")          =>    Keyword::new(KeywordKind::Subtitles),
        config.add_keyword("BIG5", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Dub", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Dubbed", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Hardsub", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Hardsubs", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("RAW", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Softsub", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Softsubs", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Sub", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Subbed", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Subtitled", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Multisub", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Multi Sub", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("Multi-Sub", Keyword::new(KeywordKind::Subtitles));
        config.add_keyword("CC", Keyword::ambiguous(KeywordKind::Subtitles));
        config.add_keyword("SDH", Keyword::ambiguous(KeywordKind::Subtitles));

        // Type
        config.add_keyword("TV", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("Movie", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("Gekijouban", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("OAD", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("OAV", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("ONA", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("OVA", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("SP", Keyword::ambiguous(KeywordKind::Type)); // e.g. "Yumeiro Patissiere SP Professional"
        config.add_keyword("Special", Keyword::ambiguous(KeywordKind::Type));
        config.add_keyword("Specials", Keyword::ambiguous(KeywordKind::Type));

        // Video
        //
        // Color depth
        config.add_keyword("8bit", Keyword::new(KeywordKind::VideoColorDepth));
        config.add_keyword("8-bit", Keyword::new(KeywordKind::VideoColorDepth));
        config.add_keyword("10bit", Keyword::new(KeywordKind::VideoColorDepth));
        config.add_keyword("10bits", Keyword::new(KeywordKind::VideoColorDepth));
        config.add_keyword("10-bit", Keyword::new(KeywordKind::VideoColorDepth));
        config.add_keyword("10-bits", Keyword::new(KeywordKind::VideoColorDepth));
        // Codec
        config.add_keyword("AV1", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("AVC", Keyword::new(KeywordKind::VideoCodec));
        // UncasedStr::new("DivX")         =>    Keyword::new(KeywordKind::VideoCodec),  // @Warning: Duplicate
        config.add_keyword("DivX5", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("DivX6", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("H.264", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("H.265", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("X.264", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("H264", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("H265", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("X264", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("X265", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("HEVC", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("HEVC2", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("Xvid", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("HDR", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("DV", Keyword::new(KeywordKind::VideoCodec));
        config.add_keyword("Dolby Vision", Keyword::new(KeywordKind::VideoCodec));
        // Format
        // UncasedStr::new("AVI")          =>    Keyword::new(KeywordKind::VideoFormat),  // @Warning: Duplicate
        // UncasedStr::new("RMVB")         =>    Keyword::new(KeywordKind::VideoFormat),  // @Warning: Duplicate
        // UncasedStr::new("WMV")          =>    Keyword::new(KeywordKind::VideoFormat),  // @Warning: Duplicate
        config.add_keyword("WMV3", Keyword::new(KeywordKind::VideoFormat));
        config.add_keyword("WMV9", Keyword::new(KeywordKind::VideoFormat));
        // Frame rate
        config.add_keyword("23.976FPS", Keyword::new(KeywordKind::VideoFrameRate));
        config.add_keyword("24FPS", Keyword::new(KeywordKind::VideoFrameRate));
        config.add_keyword("29.97FPS", Keyword::new(KeywordKind::VideoFrameRate));
        config.add_keyword("30FPS", Keyword::new(KeywordKind::VideoFrameRate));
        config.add_keyword("60FPS", Keyword::new(KeywordKind::VideoFrameRate));
        config.add_keyword("120FPS", Keyword::new(KeywordKind::VideoFrameRate));
        // Profile
        config.add_keyword("Hi10", Keyword::new(KeywordKind::VideoProfile));
        config.add_keyword("Hi10p", Keyword::new(KeywordKind::VideoProfile));
        config.add_keyword("Hi444", Keyword::new(KeywordKind::VideoProfile));
        config.add_keyword("Hi444P", Keyword::new(KeywordKind::VideoProfile));
        config.add_keyword("Hi444PP", Keyword::new(KeywordKind::VideoProfile));
        // Quality
        config.add_keyword("HD", Keyword::new(KeywordKind::VideoQuality));
        config.add_keyword("SD", Keyword::new(KeywordKind::VideoQuality));
        config.add_keyword("HQ", Keyword::new(KeywordKind::VideoQuality));
        config.add_keyword("LQ", Keyword::new(KeywordKind::VideoQuality));
        // Resolution
        config.add_keyword("480p", Keyword::unbounded(KeywordKind::VideoResolution));
        config.add_keyword("720p", Keyword::unbounded(KeywordKind::VideoResolution));
        config.add_keyword("1080p", Keyword::unbounded(KeywordKind::VideoResolution));
        config.add_keyword("1440p", Keyword::unbounded(KeywordKind::VideoResolution));
        config.add_keyword("2160p", Keyword::unbounded(KeywordKind::VideoResolution));
        config.add_keyword("4K", Keyword::new(KeywordKind::VideoResolution));

        // Volume
        config.add_keyword("Vol", Keyword::new(KeywordKind::Volume));
        config.add_keyword("Volume", Keyword::new(KeywordKind::Volume));
        config
    }

    pub(crate) fn add_keyword<S: AsRef<str>>(&mut self, keyword: S, kind: Keyword) {
        let keyword = keyword.as_ref().to_lowercase();
        self.keywords.insert(keyword, kind);
    }

    pub(crate) fn get_keyword<S: AsRef<str>>(&self, keyword: S) -> Option<&Keyword> {
        let keyword = keyword.as_ref().to_lowercase();
        self.keywords.get(&keyword)
    }

    pub(crate) fn keywords(&self) -> &HashMap<String, Keyword> {
        &self.keywords
    }
}
