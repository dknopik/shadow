use syscall_logger::log_syscall;

use crate::cshadow;
use crate::host::syscall::handler::{SyscallContext, SyscallHandler};
use crate::host::syscall::type_formatting::SyscallStringArg;
use crate::host::syscall_types::{SysCallArgs, SyscallResult};

impl SyscallHandler {
    #[log_syscall(/* rv */ libc::c_int, /* pathname */ SyscallStringArg,
                  /* flags */ nix::fcntl::OFlag, /* mode */ nix::sys::stat::Mode)]
    pub fn open(ctx: &mut SyscallContext, _args: &SysCallArgs) -> SyscallResult {
        Self::legacy_syscall(cshadow::syscallhandler_open, ctx)
    }

    #[log_syscall(/* rv */ libc::c_int, /* dirfd */ libc::c_int, /* pathname */ SyscallStringArg,
                  /* flags */ nix::fcntl::OFlag, /* mode */ nix::sys::stat::Mode)]
    pub fn openat(ctx: &mut SyscallContext, _args: &SysCallArgs) -> SyscallResult {
        Self::legacy_syscall(cshadow::syscallhandler_openat, ctx)
    }
}
