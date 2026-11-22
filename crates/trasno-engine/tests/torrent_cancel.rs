//! Cancelar unha descarga de torrent que non avanza (ficheiro á parte: a cancelación é global e non debe pisar outras probas).

use librqbit::spawn_utils::BlockingSpawner;
use librqbit::{create_torrent, CreateTorrentOptions};
use trasno_engine::{error::EngineError, ffmpeg, torrent};

#[test]
fn cancelar_para_un_torrent_sen_sementadores_e_non_deixa_restos() {
    let d = tempfile::tempdir().unwrap();
    let data = d.path().join("morto");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("a.bin"), vec![7u8; 100_000]).unwrap();
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build().unwrap();
    let file = d.path().join("morto.torrent");
    let f = file.clone();
    rt.block_on(async move {
        let made = create_torrent(&data, CreateTorrentOptions { name: Some("morto"), trackers: vec![], piece_length: Some(32 * 1024) }, &BlockingSpawner::new(1)).await.unwrap();
        std::fs::write(f, made.as_bytes().unwrap()).unwrap();
    });
    let dest = d.path().join("destino");
    ffmpeg::begin_job();
    let src = file.to_string_lossy().to_string();
    let dest2 = dest.clone();
    let t = std::thread::spawn(move || torrent::download(&src, &dest2, None));
    std::thread::sleep(std::time::Duration::from_millis(2500));
    ffmpeg::cancel_current();
    let r = t.join().unwrap();
    assert!(matches!(r, Err(EngineError::Cancelled)), "{r:?}");
    assert!(std::fs::read_dir(&dest).unwrap().next().is_none(), "sen cartafol temporal nin ficheiros soltos");
}
