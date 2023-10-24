use std::{
    io::{self, BufRead, Write},
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

pub fn stdio_transport() -> (
    Sender<String>,
    Receiver<String>,
    [JoinHandle<io::Result<()>>; 2],
) {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open("e:/projects/temp/new text document.txt")
        .unwrap();
    let mut file2 = file.try_clone().unwrap();
    let (reader_tx, reader_rx) = mpsc::channel();
    let reader_thread = thread::spawn(move || {
        io::stdin().lock().lines().try_for_each(|line| {
            let line = line?;

            let _ = file2.write_all(b"__________stdin___________\n");
            let _ = file2.write_all(line.as_bytes());
            let _ = file2.write_all(b"\n__________________________\n");
            if !line.is_empty() {
                let _ = reader_tx.send(line);
            }
            Ok(())
        })
    });

    let (writer_tx, writer_rx) = mpsc::channel();
    let writer_thread = thread::spawn(move || {
        let mut stdout = io::stdout().lock();
        writer_rx.into_iter().try_for_each(|line: String| {
            let _ = file.write_all(b"__________stdout___________\n");
            let _ = file.write_all(line.as_bytes());
            let _ = file.write_all(b"\n__________________________\n");
            writeln!(stdout, "{line}")
        })
    });

    (writer_tx, reader_rx, [writer_thread, reader_thread])
}
