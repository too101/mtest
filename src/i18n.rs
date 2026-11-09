//! Labels for the languages the original carries string tables for (ids 1000-12042).

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    English,
    Finnish,
    German,
    French,
    Spanish,
    Dutch,
    Swedish,
    Norwegian,
    Italian,
    Thai,
}

/// Order shown to the person: English, Thai, then the original's other languages.
pub const ALL: [Lang; 10] = [
    Lang::English,
    Lang::Thai,
    Lang::Finnish,
    Lang::German,
    Lang::French,
    Lang::Spanish,
    Lang::Dutch,
    Lang::Swedish,
    Lang::Norwegian,
    Lang::Italian,
];

pub struct Labels {
    pub name: &'static str,
    pub code: &'static str,
    pub geometry: &'static str,
    pub brightness: &'static str,
    pub high_voltage: &'static str,
    pub colors: &'static str,
    pub control_panel: &'static str,
    pub help: &'static str,
    pub convergence: &'static str,
    pub focus: &'static str,
    pub resolution: &'static str,
    pub moire: &'static str,
    pub readability: &'static str,
    pub jitter: &'static str,
    pub sound: &'static str,
    pub quit: &'static str,
    pub ok: &'static str,
    #[allow(dead_code)]
    pub cancel: &'static str,
    pub true_color: &'static str,
    pub pixels: &'static str,
}

impl Lang {
    pub fn from_code(code: &str) -> Option<Lang> {
        ALL.iter().copied().find(|l| l.labels().code == code)
    }

    pub fn labels(self) -> &'static Labels {
        match self {
            Lang::English => &EN,
            Lang::Finnish => &FI,
            Lang::German => &DE,
            Lang::French => &FR,
            Lang::Spanish => &ES,
            Lang::Dutch => &NL,
            Lang::Swedish => &SV,
            Lang::Norwegian => &NO,
            Lang::Italian => &IT,
            Lang::Thai => &TH,
        }
    }
}

impl Labels {
    /// "%d x %d pixels, %s" with "true color (%d bits)" (strings x039 and x041).
    pub fn info(&self, w: u32, h: u32, bits: u32) -> String {
        let depth = self.true_color.replace("{b}", &bits.to_string());
        self.pixels
            .replace("{w}", &w.to_string())
            .replace("{h}", &h.to_string())
            .replace("{c}", &depth)
    }
}

static EN: Labels = Labels {
    name: "English",
    code: "en",
    geometry: "Geometry",
    brightness: "Brightness and contrast",
    high_voltage: "High voltage",
    colors: "Colors",
    control_panel: "To control panel/display",
    help: "Help",
    convergence: "Convergence",
    focus: "Focus",
    resolution: "Resolution",
    moire: "Moire",
    readability: "Readability",
    jitter: "Jitter",
    sound: "Sound, starts from left and ends to right",
    quit: "Quit",
    ok: "OK",
    cancel: "Cancel",
    true_color: "true color ({b} bits)",
    pixels: "{w} x {h} pixels, {c}",
};

static FI: Labels = Labels {
    name: "Suomi",
    code: "fi",
    geometry: "Geometria",
    brightness: "Valoisuus ja kontrasti",
    high_voltage: "Suurjännite",
    colors: "Värit",
    control_panel: "Säädinpaneeli / Näyttö",
    help: "Help",
    convergence: "Konvergenssi",
    focus: "Fokus",
    resolution: "Erottelutarkkuus",
    moire: "Moire",
    readability: "Luettavuus",
    jitter: "Värinä",
    sound: "Ääni, alkaen vasemmalta päätyen oikealle",
    quit: "Lopeta",
    ok: "OK",
    cancel: "Peruuta",
    true_color: "True color ({b} bittiä)",
    pixels: "{w} x {h} pikseliä, {c}",
};

static DE: Labels = Labels {
    name: "German",
    code: "de",
    geometry: "Geometrie",
    brightness: "Helligkeit und Kontrast",
    high_voltage: "Hochspannung",
    colors: "Farben",
    control_panel: "Zu Systemsteuerung / Anzeige",
    help: "Hilfe",
    convergence: "Konvergenz",
    focus: "Schärfe",
    resolution: "Auflösung",
    moire: "Moiré",
    readability: "Lesbarkeit",
    jitter: "Flattern",
    sound: "Ton, wandert von links nach rechts",
    quit: "Beenden",
    ok: "OK",
    cancel: "Abbrechen",
    true_color: "True color ({b} bit)",
    pixels: "{w} x {h} Bildpunkte, {c}",
};

static FR: Labels = Labels {
    name: "French",
    code: "fr",
    geometry: "Géométrie",
    brightness: "Luminosité et contraste",
    high_voltage: "Haute tension",
    colors: "Couleurs",
    control_panel: "Vers le panneau de commande/Affichage",
    help: "Aide",
    convergence: "Convergence",
    focus: "Mise au point",
    resolution: "Résolution",
    moire: "Moiré",
    readability: "Lisibilité",
    jitter: "Gigue",
    sound: "Son, de gauche à droite",
    quit: "Quitter",
    ok: "OK",
    cancel: "Annuler",
    true_color: "Couleurs ({b} bits)",
    pixels: "{w} x {h} pixels, {c}",
};

static ES: Labels = Labels {
    name: "Spanish",
    code: "es",
    geometry: "Geometría",
    brightness: "Brillo y contraste",
    high_voltage: "Alto voltaje",
    colors: "Colores",
    control_panel: "Panel de control/Pantalla",
    help: "Ayuda",
    convergence: "Convergencia",
    focus: "Enfoque",
    resolution: "Resolución",
    moire: "Efecto muaré",
    readability: "Legibilidad",
    jitter: "Vibración de la imagen",
    sound: "Sonido, empieza a la izquierda y acaba a la derecha",
    quit: "Salir",
    ok: "OK",
    cancel: "Cancelar",
    true_color: "Color verdadero ({b} bits)",
    pixels: "{w} x {h} píxeles, {c}",
};

static NL: Labels = Labels {
    name: "Dutch",
    code: "nl",
    geometry: "Geometrische configuratie",
    brightness: "Helderheid en contrast",
    high_voltage: "Hoogspanning",
    colors: "Kleur",
    control_panel: "Naar Configuratiescherm / Beeldscherm",
    help: "Help",
    convergence: "Convergentie",
    focus: "Scherpte",
    resolution: "Resolutie",
    moire: "Moiré",
    readability: "Leesbaarheid",
    jitter: "Synchronisatiestoring",
    sound: "Geluid, begint links en eindigt rechts",
    quit: "Beëindigen",
    ok: "OK",
    cancel: "Annuleren",
    true_color: "Ware kleuren ({b} bits)",
    pixels: "{w} x {h} pixels, {c}",
};

static SV: Labels = Labels {
    name: "Swedish",
    code: "sv",
    geometry: "Geometri",
    brightness: "Ljusstyrka och kontrast",
    high_voltage: "Högspänning",
    colors: "Färger",
    control_panel: "Till kontrollpanel",
    help: "Hjälp",
    convergence: "Konvergens",
    focus: "Skärpa",
    resolution: "Upplösning",
    moire: "Moire",
    readability: "Läsbarhet",
    jitter: "Hoppning",
    sound: "Ljud, börjar från vänster och slutar till höger",
    quit: "Avsluta",
    ok: "OK",
    cancel: "Avbryt",
    true_color: "True color ({b} bits)",
    pixels: "{w} x {h} pikslar, {c}",
};

static NO: Labels = Labels {
    name: "Norwegian",
    code: "no",
    geometry: "Geometri",
    brightness: "Lysstyrke og kontrast",
    high_voltage: "Høyspenning",
    colors: "Farger",
    control_panel: "Til Kontrollpanel/Skjerm",
    help: "Hjelp",
    convergence: "Konvergens",
    focus: "Fokusering",
    resolution: "Oppløsning",
    moire: "Moiré",
    readability: "Lesbarhet",
    jitter: "Dirring",
    sound: "Lyd, går fra venstre til høyre side",
    quit: "Avslutt",
    ok: "OK",
    cancel: "Avbryt",
    true_color: "Millioner av farger ({b} biters)",
    pixels: "{w} x {h} piksler, {c}",
};

static IT: Labels = Labels {
    name: "Italian",
    code: "it",
    geometry: "Geometria",
    brightness: "Luminosità e contrasto",
    high_voltage: "Alta tensione",
    colors: "Colori",
    control_panel: "Al Pannello di controllo/Video",
    help: "Guida",
    convergence: "Convergenza",
    focus: "Messa a fuoco",
    resolution: "Risoluzione",
    moire: "Effetto Moire",
    readability: "Leggibilità",
    jitter: "Distorsione",
    sound: "Suono, inizio a sinistra e fine a destra",
    quit: "Esci",
    ok: "OK",
    cancel: "Annulla",
    true_color: "True color ({b} bit)",
    pixels: "{w} x {h} pixel, {c}",
};

/// Thai: not in the original (added for this version); written for this app.
static TH: Labels = Labels {
    name: "ไทย (Thai)",
    code: "th",
    geometry: "รูปทรงเรขาคณิต",
    brightness: "ความสว่าง และคอนทราสต์",
    high_voltage: "ไฟฟ้าแรงสูง",
    colors: "สี",
    control_panel: "ไปที่ การตั้งค่าจอแสดงผล",
    help: "วิธีใช้",
    convergence: "การบรรจบของสี",
    focus: "โฟกัส",
    resolution: "ความละเอียด",
    moire: "มัวเร",
    readability: "การอ่านง่าย",
    jitter: "ภาพสั่น",
    sound: "เสียง เริ่มจากซ้ายและจบที่ขวา",
    quit: "ออก",
    ok: "ตกลง",
    cancel: "ยกเลิก",
    true_color: "สีจริง ({b} บิต)",
    pixels: "{w} x {h} พิกเซล, {c}",
};
