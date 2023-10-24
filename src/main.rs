use cheetah::{handle_cmd, registrar::WatcherRegistrar};

fn main() {
    let mut registrar = WatcherRegistrar::default();
    let (writer_tx, reader_rx, io_threads) = cheetah::transport::stdio_transport();

    while let Ok(line) = reader_rx.recv() {
        if let Err(e) = handle_cmd(line, writer_tx.clone(), &mut registrar) {
            let _ = writer_tx.send(format!("{e:?}"));
        }
    }

    for handle in io_threads {
        let _ = handle.join();
    }
}
