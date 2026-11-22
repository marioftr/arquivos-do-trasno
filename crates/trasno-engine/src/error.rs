use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("non se atopou o ficheiro de entrada: {0}")]
    InputNotFound(PathBuf),

    #[error("formato de orixe non recoñecido para '{0}'")]
    UnknownSourceFormat(PathBuf),

    #[error("non se soporta a conversión de '{from}' a '{to}'")]
    UnsupportedConversion { from: String, to: String },

    #[error("erro de entrada/saída: {0}")]
    Io(#[from] std::io::Error),

    #[error("erro decodificando ou codificando a imaxe: {0}")]
    Image(#[from] image::ImageError),

    #[error("erro manipulando o PDF: {0}")]
    Pdf(String),

    #[error("A páxina {page} non existe: o PDF só ten {total}.")]
    PageMissing { page: u32, total: u32 },

    #[error("non se puido ler o ficheiro de texto: {0}")]
    TextReadFailed(String),

    #[error("non se puido extraer o texto: {0}")]
    TextExtractFailed(String),

    #[error("erro manipulando o modelo 3D: {0}")]
    Model3d(String),

    #[error("De momento só convirto mallas sinxelas ({formats}); de «{from}» a «{to}» aínda non.")]
    Model3dUnsupported { formats: String, from: String, to: String },

    #[error("Non atopo ffmpeg neste equipo. Instálao (por exemplo con «winget install ffmpeg») ou pon ffmpeg.exe e ffprobe.exe ao carón da app.")]
    FfmpegNotFound,

    #[error("traballo cancelado")]
    Cancelled,

    #[error("non hai Microsoft Office nin LibreOffice para facer esta conversión: {0}")]
    OfficeMissing(String),

    #[error("o programa de Office non puido facer a conversión: {0}")]
    OfficeFailed(String),

    #[error("ffmpeg rematou con erro (código {code:?}): {stderr}")]
    FfmpegFailed { code: Option<i32>, stderr: String },

    #[error("falta unha extensión opcional: {0}")]
    ExtensionMissing(&'static str),

    #[error("non hai voz que transcribir")]
    NoSpeech,

    #[error("{program} non puido rematar: {detail}")]
    ExternalFailed { program: &'static str, detail: String },

    /// Unha ligazón que non se pode descargar por un motivo coñecido («drm», «private», «login»…).
    #[error("non se pode descargar ({0})")]
    Unavailable(&'static str),

    #[error("verificación de integridade fallida: {0}")]
    IntegrityCheck(String),

    #[error("O resultado quedaría baleiro ou estragado, así que cancelei a operación e non toquei nada.")]
    EmptyOrCorruptOutput,

    #[error("o ficheiro de destino xa existe e non se permitiu sobrescribir: {0}")]
    DestinationExists(PathBuf),
}

pub type Result<T> = std::result::Result<T, EngineError>;

impl EngineError {
    /// Forma que viaxa á interface: `@@{"k": plantilla en galego, "p": parámetros}`.
    /// A interface busca a plantilla no idioma elixido; a clave `dk` indica que o
    /// parámetro `detail` é tamén unha frase fixa que se pode traducir.
    pub fn wire(&self) -> String {
        use serde_json::json;
        let (k, p) = match self {
            Self::InputNotFound(p) => ("Non atopo o ficheiro «{path}»; quizais se moveu ou se borrou.", json!({ "path": p.display().to_string() })),
            Self::UnknownSourceFormat(p) => ("Non recoñezo o formato de «{path}».", json!({ "path": p.display().to_string() })),
            Self::UnsupportedConversion { from, to } => ("Aínda non sei converter de «{from}» a «{to}».", json!({ "from": from, "to": to })),
            Self::Io(e) => ("Tiven un problema co disco: {detail}", json!({ "detail": e.to_string() })),
            Self::Image(e) => ("Non puiden ler nin escribir a imaxe: {detail}", json!({ "detail": e.to_string() })),
            Self::Pdf(m) => ("Atopei un problema co PDF: {detail}", json!({ "detail": m, "dk": true })),
            Self::PageMissing { page, total } => ("A páxina {page} non existe: o PDF só ten {total}.", json!({ "page": page, "total": total })),
            Self::TextReadFailed(m) => ("Non puiden ler o ficheiro de texto: {detail}", json!({ "detail": m })),
            Self::TextExtractFailed(m) => ("Non puiden sacar o texto do PDF: {detail}", json!({ "detail": m })),
            Self::Model3d(m) => ("Atopei un problema co modelo 3D: {detail}", json!({ "detail": m, "dk": true })),
            Self::Model3dUnsupported { formats, from, to } => (
                "De momento só convirto mallas sinxelas ({formats}); de «{from}» a «{to}» aínda non.",
                json!({ "formats": formats, "from": from, "to": to }),
            ),
            Self::Cancelled => ("Cancelei o traballo.", json!({})),
            Self::OfficeMissing(m) if !m.is_empty() => ("{detail}", json!({ "detail": m, "dk": true })),
            Self::OfficeMissing(_) => ("Para converter documentos de Office fai falta Microsoft Office ou LibreOffice, e non atopo ningún dos dous neste equipo.", json!({})),
            Self::OfficeFailed(m) => ("O programa de Office non puido facer a conversión: {detail}", json!({ "detail": m })),
            Self::FfmpegNotFound => ("Non atopo ffmpeg neste equipo. Instálao (por exemplo con «winget install ffmpeg») ou pon ffmpeg.exe e ffprobe.exe ao carón da app.", json!({})),
            Self::FfmpegFailed { code, stderr } => (
                "ffmpeg non puido rematar (código {code}). Detalle técnico: {stderr}",
                json!({ "code": code.map(|c| c.to_string()).unwrap_or_else(|| "?".into()), "stderr": stderr }),
            ),
            Self::ExtensionMissing(what) => (
                match *what {
                    "whisper" => "Para transcribir fai falta whisper.cpp, que non está instalado. Descárgase en Axustes → Extensións.",
                    "whisper-model" => "Para transcribir fai falta o modelo de voz, que non está descargado. Descárgase en Axustes → Extensións.",
                    "cutout-model" => "Para quitar o fondo fai falta o modelo de imaxe, que non está descargado. Descárgase en Axustes → Extensións.",
                    _ => "Para descargar por ligazón fai falta yt-dlp, que non está instalado. Descárgase en Axustes → Extensións.",
                },
                json!({}),
            ),
            Self::NoSpeech => ("Non atopei voz que transcribir neste ficheiro.", json!({})),
            Self::ExternalFailed { program, detail } => ("{program} non puido rematar. Detalle técnico: {detail}", json!({ "program": program, "detail": detail })),
            Self::Unavailable("drm") => ("Non podo descargar este vídeo: está protexido con DRM e non hai forma de gardalo.", json!({})),
            Self::Unavailable("private") => ("Non podo descargar este vídeo: é privado.", json!({})),
            Self::Unavailable("age") => ("Non podo descargar este vídeo: ten restrición de idade e pide iniciar sesión.", json!({})),
            Self::Unavailable("login") => ("Non podo descargar este vídeo: o sitio pide iniciar sesión para velo.", json!({})),
            Self::Unavailable("geo") => ("Non podo descargar este vídeo: non está dispoñible neste país.", json!({})),
            Self::Unavailable("live") => ("Non podo descargar este vídeo: é unha emisión que aínda non empezou.", json!({})),
            Self::Unavailable("gone") => ("Non podo descargar este vídeo: xa non existe ou foi retirado.", json!({})),
            Self::Unavailable("unsupported") => ("Non podo descargar desta ligazón: o sitio non é compatible ou non leva a un vídeo.", json!({})),
            Self::Unavailable(_) => ("Non puiden conectar co sitio. Revisa a conexión e volve probar.", json!({})),
            Self::IntegrityCheck(m) => ("Parei o traballo por seguridade: {detail}", json!({ "detail": m, "dk": true })),
            Self::EmptyOrCorruptOutput => ("O resultado quedaría baleiro ou estragado, así que cancelei a operación e non toquei nada.", json!({})),
            Self::DestinationExists(p) => ("O ficheiro de destino xa existe e non me deixaches sobrescribilo: {path}", json!({ "path": p.display().to_string() })),
        };
        format!("@@{}", json!({ "k": k, "p": p }))
    }
}
