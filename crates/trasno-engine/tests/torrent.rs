//! Torrents de punta a punta sen internet: un sementador local (outra sesión de librqbit en 127.0.0.1) e a descarga do motor, co avance,
//! a elección de ficheiros, o cancelar e o destino sen pisar nada.

use librqbit::spawn_utils::BlockingSpawner;
use librqbit::{create_torrent, AddTorrent, AddTorrentOptions, CreateTorrentOptions, ListenerOptions, Session, SessionOptions};
use std::net::SocketAddr;
use std::path::Path;
use trasno_engine::torrent;

struct Seeder {
    _rt: tokio::runtime::Runtime,
    addr: SocketAddr,
    torrent_file: std::path::PathBuf,
}

/// Un torrent de dous ficheiros (`proba/a.bin` e `proba/b.txt`) e un sementador local que o ten completo.
fn seeder(root: &Path) -> Seeder {
    let data = root.join("proba");
    std::fs::create_dir_all(&data).unwrap();
    let a: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
    std::fs::write(data.join("a.bin"), &a).unwrap();
    std::fs::write(data.join("b.txt"), "ola torrent").unwrap();
    let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build().unwrap();
    let torrent_file = root.join("proba.torrent");
    let tf = torrent_file.clone();
    let addr = rt.block_on(async move {
        let spawner = BlockingSpawner::new(2);
        let made = create_torrent(&data, CreateTorrentOptions { name: Some("proba"), trackers: vec![], piece_length: Some(32 * 1024) }, &spawner).await.unwrap();
        std::fs::write(&tf, made.as_bytes().unwrap()).unwrap();
        let session = Session::new_with_opts(
            data.parent().unwrap().join("sementador"),
            SessionOptions {
                dht: None,
                disable_local_service_discovery: true,
                listen: Some(ListenerOptions { listen_addr: "127.0.0.1:0".parse().unwrap(), ..Default::default() }),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        // O contido xa está en `root/proba`: o sementador comproba as pezas e queda completo.
        let handle = session
            .add_torrent(AddTorrent::from_bytes(made.as_bytes().unwrap()), Some(AddTorrentOptions { output_folder: Some(data.to_string_lossy().to_string()), overwrite: true, ..Default::default() }))
            .await
            .unwrap()
            .into_handle()
            .unwrap();
        handle.wait_until_completed().await.unwrap();
        // O objecto Session ten que seguir vivo: pérdese co runtime, que se garda en `Seeder`.
        let addr = session.listen_addr().expect("o sementador escoita");
        std::mem::forget(session);
        addr
    });
    Seeder { _rt: rt, addr, torrent_file }
}

#[test]
fn le_os_ficheiros_dun_torrent_sen_rede() {
    let d = tempfile::tempdir().unwrap();
    let s = seeder(d.path());
    let info = torrent::inspect(&s.torrent_file.to_string_lossy()).unwrap();
    assert_eq!(info.name, "proba");
    assert_eq!(info.files.len(), 2);
    assert_eq!(info.total, 300_000 + 11);
    assert!(info.files.iter().any(|f| f.path == "a.bin" && f.size == 300_000));
    assert!(torrent::inspect("C:/nunca/existe.torrent").is_err());
}

#[test]
fn descarga_un_torrent_completo_e_so_os_ficheiros_elixidos() {
    let d = tempfile::tempdir().unwrap();
    let s = seeder(d.path());
    let src = s.torrent_file.to_string_lossy().to_string();
    let info = torrent::inspect(&src).unwrap();

    // Todo: queda un cartafol «proba» no destino, cos bytes bos.
    let dest = d.path().join("destino");
    let r = torrent::download_with(&src, &dest, None, &[s.addr]).unwrap();
    assert_eq!(r.outputs.len(), 1);
    assert_eq!(r.outputs[0], dest.join("proba"));
    assert_eq!(std::fs::read(dest.join("proba/b.txt")).unwrap(), b"ola torrent");
    assert_eq!(std::fs::read(dest.join("proba/a.bin")).unwrap().len(), 300_000);
    assert_eq!(r.bytes_written, info.total);
    // Nada temporal queda no destino.
    assert!(std::fs::read_dir(&dest).unwrap().flatten().all(|e| !e.file_name().to_string_lossy().starts_with(".trasno-")));

    // Outra vez ao mesmo sitio: non pisa nada, o novo vai a «proba (2)».
    let again = torrent::download_with(&src, &dest, None, &[s.addr]).unwrap();
    assert_eq!(again.outputs[0], dest.join("proba (2)"));
    assert!(dest.join("proba").join("b.txt").is_file());

    // Só un ficheiro.
    let b = info.files.iter().find(|f| f.path == "b.txt").unwrap().index;
    let dest2 = d.path().join("destino2");
    let one = torrent::download_with(&src, &dest2, Some(vec![b]), &[s.addr]).unwrap();
    assert_eq!(one.bytes_written, 11);
    assert_eq!(std::fs::read(dest2.join("proba/b.txt")).unwrap(), b"ola torrent");
    assert!(!dest2.join("proba/a.bin").exists(), "o ficheiro que non se elixiu non se baixa");
    assert!(torrent::download_with(&src, &dest2, Some(vec![]), &[s.addr]).is_err());
}
