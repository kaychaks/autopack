use crate::{autopack::AutoPack, runtime::Runtime};
use std::fs::{self, OpenOptions};
use tempfile::Builder;

#[test]
fn app_state_save_load() {
    let runtime_dir = Builder::new()
        .tempdir()
        .expect("expecting a temp file to be created");
    let runtime = Runtime::builder(runtime_dir.path())
        .dir(false)
        .expect("")
        .build();
    let app = AutoPack::new(Some(runtime.clone()));

    app.save(None).expect("failed save");
    let saved_app = AutoPack::load(Some(&runtime.dir())).expect("failed load");

    assert_eq!(app, saved_app);
}

#[test]
#[should_panic]
fn app_state_bad_content() {
    use std::io::Write;

    let app = AutoPack::default();
    let save_path = Builder::new()
        .tempdir()
        .expect("expecting a temp file to be created");

    let s = app.save(Some(save_path.path())).expect("failed save");
    // let st = StateFiles::new(&s.state_content).expect("failed to parse the state files");
    let mut f = OpenOptions::new()
        .append(true)
        .open(s.state_content.clone())
        .expect("failed opening state file");
    writeln!(f, "A new line").expect("failed to write to state file");

    AutoPack::load(Some(s.state_content.as_path())).expect("failed load");
}

#[test]
#[should_panic]
fn app_state_bad_hash() {
    let app = AutoPack::default();
    let save_path = Builder::new()
        .tempdir()
        .expect("expecting a temp file to be created");

    let s = app.save(Some(save_path.path())).expect("failed save");
    // let st = StateFiles::new(&s).expect("failed to parse the state files");

    let lines = fs::read_to_string(s.checksum.clone()).expect("failed reading checksum");
    let ls = lines.lines().collect::<Vec<_>>();
    let ls1 = ls[0];
    let mut ls11: Vec<_> = ls1.chars().take(ls1.len() - 2).collect();
    ls11.push('0');
    ls11.push('1');

    let x = ls11.into_iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let lss = format!(
        r"{}
    {}",
        &x.join(""),
        ls[1]
    );
    fs::write(s.checksum, lss).expect("failed to write checksum");

    AutoPack::load(Some(s.state_content.as_path())).expect("failed load");
}

#[test]
#[should_panic]
fn app_state_bad_nonce() {
    let app = AutoPack::default();
    let save_path = Builder::new()
        .tempdir()
        .expect("expecting a temp file to be created");

    let s = app.save(Some(save_path.path())).expect("failed save");
    // let st = StateFiles::new(&s).expect("failed to parse the state files");

    let lines = fs::read_to_string(s.checksum.clone()).expect("failed reading checksum");
    let ls = lines.lines().collect::<Vec<_>>();
    let ls1 = ls[1];
    let mut ls11: Vec<_> = ls1.chars().take(ls1.len() - 2).collect();
    ls11.push('0');
    ls11.push('0');

    let x = ls11.into_iter().map(|x| x.to_string()).collect::<Vec<_>>();

    let lss = format!(
        r"{}
    {}",
        ls[0],
        &x.join("")
    );
    fs::write(s.checksum, lss).expect("failed to write checksum");

    AutoPack::load(Some(s.state_content.as_path())).expect("failed load");
}
