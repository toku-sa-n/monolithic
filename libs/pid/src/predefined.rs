use super::Pid;

pub const IDLE: Pid = Pid::new(0);
pub const INIT: Pid = Pid::new(1);
pub const SYSPROC: Pid = Pid::new(2);
pub const TTY: Pid = Pid::new(3);
pub const TEST_1: Pid = Pid::new(4);
pub const TEST_2: Pid = Pid::new(5);
