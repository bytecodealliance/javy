mod runner;

use runner::Runner;

#[test]
fn test_fd_write() {
    let input = rmp_serde::to_vec(&1337_u32)
        .unwrap();

    let result = Runner::new("fd_write.js")
        .exec(input)
        .unwrap();

    let output = rmp_serde::from_slice::<u32>(&result)
        .unwrap();

    assert_eq!(1337, output);
}
