mod runner;

use runner::{StoreContext, Runner};

#[test]
fn test_fd_write() {
    let input = rmp_serde::to_vec(&1337_u32)
        .unwrap();

    let mut ctx = StoreContext::from(input);
    let stdout_pipe = ctx.pipe_stdout();

    let result = Runner::new("fd_write.js")
        .exec(ctx)
        .unwrap();

    let output = rmp_serde::from_slice::<u32>(&result)
        .unwrap();

    assert_eq!(1337, output);
    assert_eq!(b"hello world\n", stdout_pipe.try_into_inner().unwrap().get_ref().as_slice());
}
