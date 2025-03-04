use std::path::{MAIN_SEPARATOR, Path};

use zero_shell::commands::{
    cat::cat, cd::cd, cp::cp, echo::echo, mkdir::mkdir, mv::mv, rm::rm, touch::touch,
};

#[test]
fn audit_question() {
    assert!(
        mkdir(&vec!["mkdir".to_string(), "new_folder1".to_string()]).is_ok(),
        "Result of making `new_folder1` should be ok"
    );

    assert!(
        mkdir(&vec!["mkdir".to_string(), "new_folder2".to_string()]).is_ok(),
        "Result of making `new_folder2` should be ok"
    );

    let dir1 = Path::new("new_folder1");
    let dir2 = Path::new("new_folder2");

    assert!(dir1.exists(), "`new_folder1` should exist after creation");
    assert!(dir2.exists(), "`new_folder2` should exist after creation");

    assert!(
        touch(&vec![
            "touch".to_string(),
            format!("new_folder1{}new_doc.txt", MAIN_SEPARATOR),
        ])
        .is_ok(),
        "Result of creating file should be ok"
    );

    assert!(
        cd(&vec!["cd".to_string(), "new_folder1".to_string()]).is_ok(),
        "Result of cd-ing into first folder should be ok"
    );

    assert!(
        echo(&vec![
            "echo".to_string(),
            "hello".to_string(),
            ">".to_string(),
            "new_doc.txt".to_string(),
        ])
        .is_ok(),
        "Result of echoing text to file should be ok"
    );

    assert!(
        cp(&vec![
            "cp".to_string(),
            "new_doc.txt".to_string(),
            "../new_folder2".to_string(),
        ])
        .is_ok(),
        "Result of copying file should be ok"
    );

    assert!(
        cd(&vec!["cd".to_string(), "..".to_string()]).is_ok(),
        "Result of cd-ing back out of first folder should be ok"
    );

    let concatenation = cat(&vec![
        "cat".to_string(),
        format!("new_folder2{}new_doc.txt", MAIN_SEPARATOR),
    ])
    .unwrap();
    assert_eq!(
        concatenation, "hello\n",
        "Text in copied file should match original"
    );

    assert!(
        mv(&vec![
            "mv".to_string(),
            "new_folder2".to_string(),
            "new_folder1".to_string(),
        ])
        .is_ok(),
        "Result of moving `new_folder2` into `new_folder1` should be ok"
    );

    assert!(
        Path::new(&format!("new_folder1{}new_folder2", MAIN_SEPARATOR)).exists(),
        "`new_folder2` should be inside `new_folder1`"
    );

    assert!(
        rm(&vec![
            "rm".to_string(),
            "-r".to_string(),
            "new_folder1".to_string(),
        ])
        .is_ok(),
        "Should remove `new_folder1` ok"
    );

    assert!(!dir1.exists(), "`new_folder1` should not exist");
    assert!(!dir2.exists(), "`new_folder2` should not exist");
}
