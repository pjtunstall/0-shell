use std::path::{MAIN_SEPARATOR, Path};

use zero_shell::commands::{
    cat::cat, cd::cd, cp::cp, echo::echo, mkdir::mkdir, mv::mv, rm::rm, touch::touch,
};

#[test]
fn audit_question() {
    assert!(
        mkdir(&vec![
            String::from("mkdir"),
            String::from("new_folder1")
        ])
        .is_ok(),
        "result of making `new_folder1` should be ok"
    );

    assert!(
        mkdir(&vec![
            String::from("mkdir"),
            String::from("new_folder2")
        ])
        .is_ok(),
        "result of making `new_folder2` should be ok"
    );

    let dir1 = Path::new("new_folder1");
    let dir2 = Path::new("new_folder2");

    assert!(dir1.exists(), "`new_folder1` should exist after creation");
    assert!(dir2.exists(), "`new_folder2` should exist after creation");

    assert!(
        touch(&vec![String::from("touch"), format!("new_folder1{}new_doc.txt", MAIN_SEPARATOR),])
        .is_ok(),
        "result of creating file should be ok"
    );

    assert!(
        cd(&vec![String::from("cd"), String::from("new_folder1")]).is_ok(),
        "result of cd-ing into first folder should be ok"
    );

    assert!(
        echo(&vec![
            String::from("echo"),
            String::from("hello"),
            String::from(">"),
            String::from("new_doc.txt"),
        ])
        .is_ok(),
        "result of echoing text to file should be ok"
    );

    assert!(
        cp(&vec![
            String::from("cp"),
            String::from("new_doc.txt"),
            String::from("../new_folder2"),
        ])
        .is_ok(),
        "result of copying file should be ok"
    );

    assert!(
        cd(&vec![String::from("cd"), String::from("..")]).is_ok(),
        "result of cd-ing back out of first folder should be ok"
    );

    let concatenation = cat(&vec![
        String::from("cat"),
        format!("new_folder2{}new_doc.txt", MAIN_SEPARATOR),
    ])
    .expect("concatenation failed");
    assert_eq!(
        concatenation, "hello\n",
        "text in copied file should match original"
    );

    assert!(
        mv(&vec![
            String::from("mv"),
            String::from("new_folder2"),
            String::from("new_folder1"),
        ])
        .is_ok(),
        "result of moving `new_folder2` into `new_folder1` should be ok"
    );

    assert!(
        Path::new(&format!("new_folder1{}new_folder2", MAIN_SEPARATOR)).exists(),
        "`new_folder2` should be inside `new_folder1`"
    );

    assert!(
        rm(&vec![
            String::from("rm"),
            String::from("-r"),
            String::from("new_folder1"),
        ])
        .is_ok(),
        "should remove `new_folder1` ok"
    );

    assert!(!dir1.exists(), "`new_folder1` should not exist");
    assert!(!dir2.exists(), "`new_folder2` should not exist");
}
