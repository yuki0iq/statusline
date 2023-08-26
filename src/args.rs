/// Parsed command line arguments
pub struct CommandLineArgs {
    /// Last command's return code
    pub(crate) ret_code: Option<u8>,
    /// Jobs currently running
    pub(crate) jobs_count: usize,
    /// Last command's elapsed tile
    pub(crate) elapsed_time: Option<u64>,
}

impl CommandLineArgs {
    /// Construct args from command line
    pub fn from_env<T: AsRef<str>>(arg: &[T]) -> CommandLineArgs {
        let ret_code = arg.get(0).map(|val| val.as_ref().parse().unwrap());
        let jobs_count = arg
            .get(1)
            .map(|val| val.as_ref().parse().unwrap_or(0))
            .unwrap_or(0);
        let elapsed_time = arg.get(2).map(|val| val.as_ref().parse().unwrap());
        CommandLineArgs {
            ret_code,
            jobs_count,
            elapsed_time,
        }
    }
}
