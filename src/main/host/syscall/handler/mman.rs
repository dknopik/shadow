use syscall_logger::log_syscall;

use crate::cshadow;
use crate::host::syscall::handler::{SyscallContext, SyscallHandler};
use crate::host::syscall_types::{SysCallArgs, SyscallResult};

impl SyscallHandler {
    #[log_syscall(/* rv */ libc::c_int, /* addr */ *const libc::c_void)]
    pub fn brk(ctx: &mut SyscallContext, _args: &SysCallArgs) -> SyscallResult {
        Self::legacy_syscall(cshadow::syscallhandler_brk, ctx)
    }

    #[log_syscall(/* rv */ *const libc::c_void, /* addr */ *const libc::c_void,
                  /* length */ libc::size_t, /* prot */ nix::sys::mman::ProtFlags,
                  /* flags */ nix::sys::mman::MapFlags, /* fd */ libc::c_int,
                  /* offset */ libc::off_t)]
    pub fn mmap(ctx: &mut SyscallContext, _args: &SysCallArgs) -> SyscallResult {
        Self::legacy_syscall(cshadow::syscallhandler_mmap, ctx)
    }

    #[log_syscall(/* rv */ *const libc::c_void, /* old_address */ *const libc::c_void,
                  /* old_size */ libc::size_t, /* new_size */ libc::size_t,
                  /* flags */ nix::sys::mman::MRemapFlags, /* new_address */ *const libc::c_void)]
    pub fn mremap(ctx: &mut SyscallContext, _args: &SysCallArgs) -> SyscallResult {
        Self::legacy_syscall(cshadow::syscallhandler_mremap, ctx)
    }

    #[log_syscall(/* rv */ libc::c_int, /* addr */ *const libc::c_void, /* length */ libc::size_t)]
    pub fn munmap(ctx: &mut SyscallContext, _args: &SysCallArgs) -> SyscallResult {
        Self::legacy_syscall(cshadow::syscallhandler_munmap, ctx)
    }

    #[log_syscall(/* rv */ libc::c_int, /* addr */ *const libc::c_void, /* len */ libc::size_t,
                  /* prot */ libc::c_int)]
    pub fn mprotect(ctx: &mut SyscallContext, _args: &SysCallArgs) -> SyscallResult {
        Self::legacy_syscall(cshadow::syscallhandler_mprotect, ctx)
    }
}
