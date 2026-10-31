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

    #[error("ffmpeg rematou con erro (código {code:?}): {stderr}")]
    FfmpegFailed { code: Option<i32>, stderr: String },

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
            Self::Io(e) => ("Houbo un problema co disco: {detail}", json!({ "detail": e.to_string() })),
            Self::Image(e) => ("A imaxe non se puido ler ou escribir: {detail}", json!({ "detail": e.to_string() })),
            Self::Pdf(m) => ("O PDF deu un problema: {detail}", json!({ "detail": m, "dk": true })),
            Self::PageMissing { page, total } => ("A páxina {page} non existe: o PDF só ten {total}.", json!({ "page": page, "total": total })),
            Self::TextReadFailed(m) => ("Non puiden ler o ficheiro de texto: {detail}", json!({ "detail": m })),
            Self::TextExtractFailed(m) => ("Non puiden sacar o texto do PDF: {detail}", json!({ "detail": m })),
            Self::Model3d(m) => ("O modelo 3D deu un problema: {detail}", json!({ "detail": m, "dk": true })),
            Self::Model3dUnsupported { formats, from, to } => (
                "De momento só convirto mallas sinxelas ({formats}); de «{from}» a «{to}» aínda non.",
                json!({ "formats": formats, "from": from, "to": to }),
            ),
            Self::FfmpegNotFound => ("Non atopo ffmpeg neste equipo. Instálao (por exemplo con «winget install ffmpeg») ou pon ffmpeg.exe e ffprobe.exe ao carón da app.", json!({})),
            Self::FfmpegFailed { code, stderr } => (
                "ffmpeg non puido rematar (código {code}). Detalle técnico: {stderr}",
                json!({ "code": code.map(|c| c.to_string()).unwrap_or_else(|| "?".into()), "stderr": stderr }),
            ),
            Self::IntegrityCheck(m) => ("Por seguridade paréi o traballo: {detail}", json!({ "detail": m, "dk": true })),
            Self::EmptyOrCorruptOutput => ("O resultado quedaría baleiro ou estragado, así que cancelei a operación e non toquei nada.", json!({})),
            Self::DestinationExists(p) => ("O ficheiro de destino xa existe e non me deixaches sobrescribilo: {path}", json!({ "path": p.display().to_string() })),
        };
        format!("@@{}", json!({ "k": k, "p": p }))
    }
}
