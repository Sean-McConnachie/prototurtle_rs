use std::io::Write;

pub trait FStore {
    fn default(p: &std::path::PathBuf) -> Self;
    fn path(&self) -> &std::path::PathBuf;
    fn save(&self) -> String;
    fn load(p: &std::path::PathBuf, d: &str) -> Self;
}

pub fn fstore_save<T: FStore>(t: &T) {
    let mut f = std::fs::File::create(t.path()).unwrap();
    f.write_all(t.save().as_bytes()).unwrap();
}

pub fn fstore_load_or_init<T: FStore>(p: &std::path::PathBuf) -> T {
    if !p.exists() {
        let t = T::default(p);
        fstore_save(&t);
        t
    } else {
        T::load(p, std::fs::read_to_string(p).unwrap().as_str())
    }
}
