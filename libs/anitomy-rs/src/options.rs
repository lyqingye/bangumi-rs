/// Options relating to the [`Tokenizer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options(u16);

impl Default for Options {
    /// The default option is to have everything enabled
    fn default() -> Self {
        Self(0b0000_0011_1111_1111)
    }
}

impl Options {
    const EPISODE: u16 = 1 << 0;
    const EPISODE_TITLE: u16 = 1 << 1;
    const FILE_CHECKSUM: u16 = 1 << 2;
    const FILE_EXTENSION: u16 = 1 << 3;
    const RELEASE_GROUP: u16 = 1 << 4;
    const SEASON: u16 = 1 << 5;
    const TITLE: u16 = 1 << 6;
    const VIDEO_RESOLUTION: u16 = 1 << 7;
    const YEAR: u16 = 1 << 8;
    const DATE: u16 = 1 << 9;

    #[inline]
    const fn has_flag(&self, val: u16) -> bool {
        (self.0 & val) == val
    }

    #[inline]
    fn toggle_flag(&mut self, val: u16, toggle: bool) {
        if toggle {
            self.0 |= val;
        } else {
            self.0 &= !val;
        }
    }

    /// Returns a bool indiciating whether to parse episodes in the filename.
    pub const fn parse_episode(&self) -> bool {
        self.has_flag(Self::EPISODE)
    }

    /// Returns a bool indiciating whether to parse episode titles in the filename.
    pub const fn parse_episode_title(&self) -> bool {
        self.has_flag(Self::EPISODE_TITLE)
    }

    /// Returns a bool indiciating whether to parse file checksums in the filename.
    pub const fn parse_file_checksum(&self) -> bool {
        self.has_flag(Self::FILE_CHECKSUM)
    }

    /// Returns a bool indiciating whether to parse file extensions in the filename.
    pub const fn parse_file_extension(&self) -> bool {
        self.has_flag(Self::FILE_EXTENSION)
    }

    /// Returns a bool indiciating whether to parse release groups in the filename.
    pub const fn parse_release_group(&self) -> bool {
        self.has_flag(Self::RELEASE_GROUP)
    }

    /// Returns a bool indiciating whether to parse seasons in the filename.
    pub const fn parse_season(&self) -> bool {
        self.has_flag(Self::SEASON)
    }

    /// Returns a bool indiciating whether to parse titles in the filename.
    pub const fn parse_title(&self) -> bool {
        self.has_flag(Self::TITLE)
    }

    /// Returns a bool indiciating whether to parse video resolutions in the filename.
    pub const fn parse_video_resolution(&self) -> bool {
        self.has_flag(Self::VIDEO_RESOLUTION)
    }

    /// Returns a bool indiciating whether to parse years in the filename.
    pub const fn parse_year(&self) -> bool {
        self.has_flag(Self::YEAR)
    }

    /// Returns a bool indiciating whether to parse dates in the filename.
    ///
    /// Only formats supported currently are `YYYY-MM-DD`
    pub const fn parse_date(&self) -> bool {
        self.has_flag(Self::DATE)
    }

    /// A builder method to toggle the option to parse episodes.
    pub fn episodes(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::EPISODE, toggle);
        self
    }

    /// A builder method to toggle the option to parse episode titles.
    pub fn episode_titles(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::EPISODE_TITLE, toggle);
        self
    }

    /// A builder method to toggle the option to parse file checksums.
    pub fn file_checksums(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::FILE_CHECKSUM, toggle);
        self
    }

    /// A builder method to toggle the option to parse file extensions.
    pub fn file_extensions(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::FILE_EXTENSION, toggle);
        self
    }

    /// A builder method to toggle the option to parse release groups.
    pub fn release_groups(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::RELEASE_GROUP, toggle);
        self
    }

    /// A builder method to toggle the option to parse seasons.
    pub fn seasons(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::SEASON, toggle);
        self
    }

    /// A builder method to toggle the option to parse titles.
    pub fn titles(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::TITLE, toggle);
        self
    }

    /// A builder method to toggle the option to parse video resolutions.
    pub fn video_resolutions(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::VIDEO_RESOLUTION, toggle);
        self
    }

    /// A builder method to toggle the option to parse years.
    pub fn years(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::YEAR, toggle);
        self
    }

    /// A builder method to toggle the option to parse years.
    pub fn dates(mut self, toggle: bool) -> Self {
        self.toggle_flag(Self::DATE, toggle);
        self
    }
}
