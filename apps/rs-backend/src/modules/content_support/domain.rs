//! Aturan murni konten CMS (DD A15.4, US-1101, US-1802), tanpa I/O.
//!
//! - Status editorial tersimpan: `draft`, `scheduled`, `archived`. "Tayang" dihitung dari status `scheduled`
//!   dan jendela `[publish_start_at, publish_end_at)` terhadap jam server (DD A15.4, US-1101 AC2).
//! - Kanal: aplikasi, situs, atau keduanya; kanal yang memuat situs wajib punya halaman tujuan (US-1802 AC1).
//! - Hanya draf yang dapat diubah atau dijadwalkan; draf dan terjadwal dapat diarsipkan. Mengubah konten
//!   yang sedang tayang = membuat versi baru lalu menayangkannya (US-1802 AC2).
//! - Halaman tetap situs (`info_page` dengan halaman situs): hanya satu versi tayang per halaman. Versi
//!   lama yang masih berjalan diberi batas akhir = mulai versi baru; versi lain yang mulai pada atau setelah
//!   versi baru adalah konflik (`409`) supaya tidak ada versi terjadwal yang tertimpa diam-diam.

use envelope::FieldError;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

/// Batas panjang judul (karakter), sama dengan `ck_cms_content_title_length`.
pub(super) const TITLE_MAX: usize = 200;
/// Batas panjang isi Markdown (karakter), sama dengan `ck_cms_content_body_length`.
pub(super) const BODY_MAX: usize = 100_000;
/// Batas panjang teks alternatif gambar (karakter).
pub(super) const ALT_TEXT_MAX: usize = 300;
/// Batas panjang kategori (karakter).
pub(super) const CATEGORY_MAX: usize = 100;
/// Batas panjang rujukan tautan (karakter).
pub(super) const LINK_REF_MAX: usize = 500;
/// Batas panjang alasan penghapusan (karakter).
pub(super) const REASON_MAX: usize = 500;

macro_rules! code_enum {
    ($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $variant:ident => $code:literal),+ $(,)? }) => {
        $(#[$m])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
        #[serde(rename_all = "snake_case")]
        pub(crate) enum $name {
            $($(#[$vm])* $variant),+
        }

        impl $name {
            /// Semua nilai, urut sesuai CHECK di migrasi.
            #[allow(dead_code)] // dipakai test dan pemeriksaan
            pub(crate) const ALL: &'static [Self] = &[$(Self::$variant),+];

            /// Kode di database dan di API.
            #[allow(dead_code)] // tidak semua enum dipakai di kedua arah
            pub(crate) const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $code),+
                }
            }

            /// Mengurai kode; kode tidak dikenal = `None`.
            #[allow(dead_code)] // tidak semua enum dipakai di kedua arah
            pub(crate) fn parse(code: &str) -> Option<Self> {
                match code {
                    $($code => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

code_enum!(
    /// Jenis konten (`ck_cms_content_content_type`).
    ContentType {
        /// Banner.
        Banner => "banner",
        /// Halaman info; di situs = halaman tetap.
        InfoPage => "info_page",
        /// Artikel edukasi atau berita.
        Article => "article",
        /// Promo informatif (tidak mengubah harga, US-1101 AC4).
        Promo => "promo",
    }
);

code_enum!(
    /// Kanal tayang (`ck_cms_content_channel`).
    Channel {
        /// Aplikasi pasien saja.
        App => "app",
        /// Situs saja.
        Site => "site",
        /// Aplikasi dan situs.
        AppAndSite => "app_and_site",
    }
);

code_enum!(
    /// Halaman situs tujuan (BR-130, `ck_cms_content_site_page`).
    SitePage {
        /// Beranda.
        Home => "home",
        /// Tentang Kami.
        About => "about",
        /// Layanan.
        Services => "services",
        /// Untuk Perusahaan.
        ForBusiness => "for_business",
        /// Rekanan.
        Partners => "partners",
        /// Artikel/berita.
        Articles => "articles",
        /// Kontak.
        Contact => "contact",
        /// Unduh aplikasi.
        DownloadApp => "download_app",
        /// Kebijakan privasi dan syarat.
        PrivacyTerms => "privacy_terms",
    }
);

code_enum!(
    /// Tujuan tautan banner (`ck_cms_content_link_target_type`).
    LinkTargetType {
        /// Artikel.
        Article => "article",
        /// Layanan.
        Service => "service",
        /// Rekanan.
        Partner => "partner",
        /// Produk.
        Product => "product",
    }
);

code_enum!(
    /// Status editorial tersimpan (`ck_cms_content_editorial_status`).
    EditorialStatus {
        /// Draf.
        Draft => "draft",
        /// Terjadwal (tayang di dalam jendela waktunya).
        Scheduled => "scheduled",
        /// Diarsipkan.
        Archived => "archived",
    }
);

code_enum!(
    /// Status tampilan yang dihitung (wireframe C14: Draf, Tayang (terjadwal), Tayang, Selesai, Diarsipkan,
    /// Terhapus).
    DisplayStatus {
        /// Draf.
        Draft => "draft",
        /// Terjadwal, waktu tayang belum tiba.
        Scheduled => "scheduled",
        /// Tayang.
        Live => "live",
        /// Waktu tayang sudah selesai.
        Ended => "ended",
        /// Diarsipkan.
        Archived => "archived",
        /// Terhapus (soft delete).
        Deleted => "deleted",
    }
);

impl Channel {
    /// Apakah kanal memuat situs.
    pub(super) const fn includes_site(self) -> bool {
        matches!(self, Self::Site | Self::AppAndSite)
    }
}

/// Status tampilan dari status tersimpan, jendela waktu, dan penanda hapus terhadap `now` (jam server).
pub(super) fn display_status(
    status: EditorialStatus,
    start: Option<OffsetDateTime>,
    end: Option<OffsetDateTime>,
    deleted: bool,
    now: OffsetDateTime,
) -> DisplayStatus {
    if deleted {
        return DisplayStatus::Deleted;
    }
    match status {
        EditorialStatus::Draft => DisplayStatus::Draft,
        EditorialStatus::Archived => DisplayStatus::Archived,
        EditorialStatus::Scheduled => match start {
            Some(s) if s > now => DisplayStatus::Scheduled,
            Some(_) if end.is_some_and(|e| e <= now) => DisplayStatus::Ended,
            Some(_) => DisplayStatus::Live,
            // Dicegah ck_cms_content_scheduled_start; bila tetap terjadi, tidak pernah dianggap tayang.
            None => DisplayStatus::Scheduled,
        },
    }
}

/// Hanya draf yang dapat diubah isinya atau gambarnya.
pub(super) fn can_edit(status: EditorialStatus) -> bool {
    status == EditorialStatus::Draft
}

/// Hanya draf yang dapat dijadwalkan.
pub(super) fn can_schedule(status: EditorialStatus) -> bool {
    status == EditorialStatus::Draft
}

/// Draf dan konten terjadwal dapat diarsipkan.
pub(super) fn can_archive(status: EditorialStatus) -> bool {
    matches!(status, EditorialStatus::Draft | EditorialStatus::Scheduled)
}

/// Isian konten yang dapat diubah lewat simpan draf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ContentFields {
    pub(super) content_type: ContentType,
    pub(super) title: String,
    pub(super) body: Option<String>,
    pub(super) category: Option<String>,
    pub(super) channel: Channel,
    pub(super) site_page: Option<SitePage>,
    pub(super) image_alt_text: Option<String>,
    pub(super) link_target_type: Option<LinkTargetType>,
    pub(super) link_target_ref: Option<String>,
}

fn too_long(field: &str, max: usize) -> FieldError {
    FieldError::new(
        field,
        "too_long",
        format!("Maksimal {max} karakter").as_str(),
    )
}

fn trimmed(v: Option<String>) -> Option<String> {
    v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

impl ContentFields {
    /// Menormalkan (trim, string kosong = tidak ada) lalu memvalidasi bentuk dan aturan kanal (KU-01: galat
    /// per isian). Aturan render aman isi Markdown divalidasi terpisah (F1-T111).
    pub(super) fn normalized(mut self) -> Result<Self, Vec<FieldError>> {
        self.title = self.title.trim().to_string();
        self.category = trimmed(self.category);
        self.image_alt_text = trimmed(self.image_alt_text);
        self.link_target_ref = trimmed(self.link_target_ref);
        self.body = self.body.filter(|b| !b.trim().is_empty());
        let mut errors = Vec::new();
        if self.title.is_empty() {
            errors.push(FieldError::new("title", "required", "Judul wajib diisi"));
        } else if self.title.chars().count() > TITLE_MAX {
            errors.push(too_long("title", TITLE_MAX));
        }
        if self
            .body
            .as_ref()
            .is_some_and(|b| b.chars().count() > BODY_MAX)
        {
            errors.push(too_long("body", BODY_MAX));
        }
        if self
            .category
            .as_ref()
            .is_some_and(|c| c.chars().count() > CATEGORY_MAX)
        {
            errors.push(too_long("category", CATEGORY_MAX));
        }
        if self
            .image_alt_text
            .as_ref()
            .is_some_and(|a| a.chars().count() > ALT_TEXT_MAX)
        {
            errors.push(too_long("image_alt_text", ALT_TEXT_MAX));
        }
        match (self.channel.includes_site(), self.site_page) {
            (true, None) => errors.push(FieldError::new(
                "site_page",
                "required",
                "Halaman situs tujuan wajib dipilih untuk kanal situs",
            )),
            (false, Some(_)) => errors.push(FieldError::new(
                "site_page",
                "not_allowed",
                "Halaman situs hanya untuk kanal situs",
            )),
            _ => {}
        }
        match (self.link_target_type, &self.link_target_ref) {
            (Some(_), None) => errors.push(FieldError::new(
                "link_target_ref",
                "required",
                "Rujukan tautan wajib diisi bila tujuan tautan dipilih",
            )),
            (None, Some(_)) => errors.push(FieldError::new(
                "link_target_type",
                "required",
                "Tujuan tautan wajib dipilih bila rujukan diisi",
            )),
            (Some(_), Some(r)) if r.chars().count() > LINK_REF_MAX => {
                errors.push(too_long("link_target_ref", LINK_REF_MAX));
            }
            _ => {}
        }
        if errors.is_empty() {
            Ok(self)
        } else {
            Err(errors)
        }
    }
}

/// Halaman tetap situs: `info_page` yang punya halaman situs tujuan.
pub(super) fn fixed_site_page(
    content_type: ContentType,
    site_page: Option<SitePage>,
) -> Option<SitePage> {
    match content_type {
        ContentType::InfoPage => site_page,
        _ => None,
    }
}

/// Alasan penghapusan (wajib, ADR-003): dinormalkan dan divalidasi.
pub(super) fn delete_reason(reason: &str) -> Result<String, Vec<FieldError>> {
    let r = reason.trim();
    if r.is_empty() {
        return Err(vec![FieldError::new(
            "delete_reason",
            "required",
            "Alasan wajib diisi",
        )]);
    }
    if r.chars().count() > REASON_MAX {
        return Err(vec![too_long("delete_reason", REASON_MAX)]);
    }
    Ok(r.to_string())
}

/// Jendela tayang yang diminta saat menjadwalkan, sudah divalidasi terhadap jam server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PublishWindow {
    pub(super) start: OffsetDateTime,
    pub(super) end: Option<OffsetDateTime>,
}

impl PublishWindow {
    /// `start = None` berarti tayang sekarang (jam server; jam perangkat tidak dipakai, ADR-017 butir 5).
    /// Waktu mulai di masa lalu dan waktu selesai yang tidak setelah waktu mulai ditolak.
    pub(super) fn new(
        start: Option<OffsetDateTime>,
        end: Option<OffsetDateTime>,
        now: OffsetDateTime,
    ) -> Result<Self, Vec<FieldError>> {
        let mut errors = Vec::new();
        let start = match start {
            Some(s) if s < now => {
                errors.push(FieldError::new(
                    "publish_start_at",
                    "in_past",
                    "Waktu mulai tayang tidak boleh di masa lalu",
                ));
                s
            }
            Some(s) => s,
            None => now,
        };
        if end.is_some_and(|e| e <= start) {
            errors.push(FieldError::new(
                "publish_end_at",
                "before_start",
                "Waktu selesai tayang harus setelah waktu mulai",
            ));
        }
        if errors.is_empty() {
            Ok(Self { start, end })
        } else {
            Err(errors)
        }
    }

    fn overlaps(&self, start: OffsetDateTime, end: Option<OffsetDateTime>) -> bool {
        let starts_before_my_end = self.end.is_none_or(|e| start < e);
        let ends_after_my_start = end.is_none_or(|e| e > self.start);
        starts_before_my_end && ends_after_my_start
    }
}

/// Versi lain halaman tetap yang sama yang berstatus terjadwal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OtherVersion {
    pub(super) id: foundation::Uuid,
    pub(super) start: OffsetDateTime,
    pub(super) end: Option<OffsetDateTime>,
}

/// Rencana penayangan halaman tetap (US-1802 AC2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum FixedPagePlan {
    /// Lanjut; versi yang disebut diberi batas akhir = mulai versi baru.
    Proceed {
        /// Versi lama yang berakhir.
        end_previous: Vec<foundation::Uuid>,
    },
    /// Ada versi lain yang mulai pada atau setelah versi baru dan tumpang tindih (409).
    Conflict,
}

/// Menyusun rencana: versi lama yang tumpang tindih dan mulai lebih dulu berakhir saat versi baru mulai.
pub(super) fn plan_fixed_page(window: &PublishWindow, others: &[OtherVersion]) -> FixedPagePlan {
    let mut end_previous = Vec::new();
    for o in others {
        if !window.overlaps(o.start, o.end) {
            continue;
        }
        if o.start >= window.start {
            return FixedPagePlan::Conflict;
        }
        end_previous.push(o.id);
    }
    FixedPagePlan::Proceed { end_previous }
}

/// Syarat menayangkan: gambar wajib punya teks alternatif (US-1802 AC5).
pub(super) fn publish_requirements(
    has_image: bool,
    image_alt_text: Option<&str>,
) -> Result<(), Vec<FieldError>> {
    if has_image && image_alt_text.is_none_or(|a| a.trim().is_empty()) {
        return Err(vec![FieldError::new(
            "image_alt_text",
            "required",
            "Teks alternatif gambar wajib diisi sebelum konten ditayangkan",
        )]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    const NOW: OffsetDateTime = datetime!(2027-03-01 01:00:00 UTC);

    fn fields(channel: Channel, page: Option<SitePage>) -> ContentFields {
        ContentFields {
            content_type: ContentType::Article,
            title: "  Judul  ".into(),
            body: Some("Isi".into()),
            category: Some("   ".into()),
            channel,
            site_page: page,
            image_alt_text: None,
            link_target_type: None,
            link_target_ref: None,
        }
    }

    #[test]
    fn codes_match_migration_check_constraints() {
        let codes = |v: Vec<&str>| v.join(",");
        assert_eq!(
            codes(ContentType::ALL.iter().map(|c| c.as_str()).collect()),
            "banner,info_page,article,promo"
        );
        assert_eq!(
            codes(Channel::ALL.iter().map(|c| c.as_str()).collect()),
            "app,site,app_and_site"
        );
        assert_eq!(
            codes(SitePage::ALL.iter().map(|c| c.as_str()).collect()),
            "home,about,services,for_business,partners,articles,contact,download_app,privacy_terms"
        );
        assert_eq!(
            codes(EditorialStatus::ALL.iter().map(|c| c.as_str()).collect()),
            "draft,scheduled,archived"
        );
        for p in SitePage::ALL {
            assert_eq!(SitePage::parse(p.as_str()), Some(*p));
        }
        assert_eq!(SitePage::parse("Home"), None);
    }

    // trace: US-1101 AC2
    #[test]
    fn display_status_follows_window_in_two_indonesian_time_zones() {
        use time::UtcOffset;
        let wib = UtcOffset::from_hms(7, 0, 0).unwrap();
        let wit = UtcOffset::from_hms(9, 0, 0).unwrap();
        // Mulai 08.00 WIB = 10.00 WIT = 01.00 UTC; instan sama apa pun zonanya (ADR-017).
        let start = datetime!(2027-03-01 08:00:00).assume_offset(wib);
        let end = datetime!(2027-03-01 12:00:00).assume_offset(wit);
        let s = EditorialStatus::Scheduled;
        let before = NOW - time::Duration::seconds(1);
        assert_eq!(
            display_status(s, Some(start), Some(end), false, before),
            DisplayStatus::Scheduled
        );
        assert_eq!(
            display_status(s, Some(start), Some(end), false, NOW),
            DisplayStatus::Live
        );
        assert_eq!(
            display_status(
                s,
                Some(start),
                Some(end),
                false,
                end.to_offset(UtcOffset::UTC)
            ),
            DisplayStatus::Ended
        );
        assert_eq!(
            display_status(s, Some(start), None, true, NOW),
            DisplayStatus::Deleted
        );
        assert_eq!(
            display_status(EditorialStatus::Archived, Some(start), None, false, NOW),
            DisplayStatus::Archived
        );
        assert_eq!(
            display_status(EditorialStatus::Draft, None, None, false, NOW),
            DisplayStatus::Draft
        );
        assert_eq!(
            display_status(s, None, None, false, NOW),
            DisplayStatus::Scheduled
        );
    }

    // trace: US-1802 AC1
    #[test]
    fn site_channel_requires_page_and_app_channel_rejects_page() {
        let ok = fields(Channel::AppAndSite, Some(SitePage::Articles))
            .normalized()
            .unwrap();
        assert_eq!(ok.title, "Judul");
        assert_eq!(ok.category, None);
        let e = fields(Channel::Site, None).normalized().unwrap_err();
        assert_eq!(e[0].field, "site_page");
        let e = fields(Channel::App, Some(SitePage::Home))
            .normalized()
            .unwrap_err();
        assert_eq!(e[0].code, "not_allowed");
    }

    #[test]
    fn lengths_and_link_pairs_are_validated() {
        let mut f = fields(Channel::App, None);
        f.title = "x".repeat(TITLE_MAX + 1);
        f.link_target_type = Some(LinkTargetType::Service);
        let e = f.normalized().unwrap_err();
        let names: Vec<&str> = e.iter().map(|x| x.field.as_str()).collect();
        assert_eq!(names, ["title", "link_target_ref"]);
        let mut f = fields(Channel::App, None);
        f.title = "   ".into();
        assert_eq!(f.normalized().unwrap_err()[0].code, "required");
        assert!(delete_reason("  ").is_err());
        assert_eq!(delete_reason(" usang ").unwrap(), "usang");
        assert!(delete_reason(&"x".repeat(REASON_MAX + 1)).is_err());
    }

    #[test]
    fn only_drafts_are_editable_and_schedulable() {
        assert!(can_edit(EditorialStatus::Draft));
        assert!(!can_edit(EditorialStatus::Scheduled));
        assert!(!can_schedule(EditorialStatus::Archived));
        assert!(can_archive(EditorialStatus::Scheduled));
        assert!(!can_archive(EditorialStatus::Archived));
    }

    #[test]
    fn publish_window_rejects_past_start_and_inverted_end() {
        let w = PublishWindow::new(None, None, NOW).unwrap();
        assert_eq!(w.start, NOW);
        let e = PublishWindow::new(Some(NOW - time::Duration::minutes(1)), None, NOW).unwrap_err();
        assert_eq!(e[0].code, "in_past");
        let e = PublishWindow::new(Some(NOW), Some(NOW), NOW).unwrap_err();
        assert_eq!(e[0].field, "publish_end_at");
    }

    // trace: US-1802 AC2
    #[test]
    fn fixed_page_ends_previous_version_and_rejects_later_one() {
        let id = |n: u128| foundation::Uuid::from_u128(n);
        let h = time::Duration::hours(1);
        let w = PublishWindow::new(Some(NOW + h), None, NOW).unwrap();
        let live = OtherVersion {
            id: id(1),
            start: NOW - h,
            end: None,
        };
        let ended = OtherVersion {
            id: id(2),
            start: NOW - h * 3,
            end: Some(NOW - h),
        };
        assert_eq!(
            plan_fixed_page(&w, &[live, ended]),
            FixedPagePlan::Proceed {
                end_previous: vec![id(1)]
            }
        );
        let later = OtherVersion {
            id: id(3),
            start: NOW + h * 2,
            end: None,
        };
        assert_eq!(plan_fixed_page(&w, &[later]), FixedPagePlan::Conflict);
        // Versi baru berbatas akhir sebelum versi lain mulai: tidak tumpang tindih.
        let bounded = PublishWindow::new(Some(NOW + h), Some(NOW + h * 2), NOW).unwrap();
        assert_eq!(
            plan_fixed_page(&bounded, &[later]),
            FixedPagePlan::Proceed {
                end_previous: vec![]
            }
        );
        assert_eq!(
            fixed_site_page(ContentType::InfoPage, Some(SitePage::About)),
            Some(SitePage::About)
        );
        assert_eq!(
            fixed_site_page(ContentType::Article, Some(SitePage::Articles)),
            None
        );
    }

    // trace: US-1802 AC5
    #[test]
    fn image_without_alt_text_cannot_be_published() {
        assert!(publish_requirements(false, None).is_ok());
        assert!(publish_requirements(true, Some("Foto")).is_ok());
        let e = publish_requirements(true, Some("  ")).unwrap_err();
        assert_eq!(e[0].field, "image_alt_text");
        assert!(publish_requirements(true, None).is_err());
    }
}
