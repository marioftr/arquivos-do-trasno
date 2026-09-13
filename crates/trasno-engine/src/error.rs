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

    #[error("erro manipulando o modelo 3D: {0}")]
    Model3d(String),

    #[error("non se atopou ffmpeg no sistema; instálao ou indica a súa ruta")]
    FfmpegNotFound,

    #[error("ffmpeg rematou con erro (código {code:?}): {stderr}")]
    FfmpegFailed { code: Option<i32>, stderr: String },

    #[error("verificación de integridade fallida: {0}")]
    IntegrityCheck(String),

    #[error("a saída quedaría baleira ou corrompida, cancelouse a operación")]
    EmptyOrCorruptOutput,

    #[error("o ficheiro de destino xa existe e non se permitiu sobrescribir: {0}")]
    DestinationExists(PathBuf),
}

pub type Result<T> = std::result::Result<T, EngineError>;
