pub trait OpenFile {
    fn open_file(&self) -> Result<io::BufWriter<fs::File>>;
}

impl<'a> OpenFile for &'a str {
    fn open_file(&self) -> Result<io::BufWriter<fs::File>> {
        let rel_path = self;

        log::trace!("opening {}", rel_path);

        let file = fs::File::create(&rel_path);
        let file = te!(file, f!("path: {}", rel_path));
        let file = io::BufWriter::new(file);

        Ok(file)
    }
}

use super::*;
