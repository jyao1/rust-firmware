// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

#[allow(dead_code)]
#[repr(packed)]
pub struct ScratchRegisters {
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rsi: usize,
    pub rdi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

impl ScratchRegisters {
    pub fn dump(&self) {
        log::info!("RAX:   {:>016X}\n", { self.rax });
        log::info!("RCX:   {:>016X}\n", { self.rcx });
        log::info!("RDX:   {:>016X}\n", { self.rdx });
        log::info!("RDI:   {:>016X}\n", { self.rdi });
        log::info!("RSI:   {:>016X}\n", { self.rsi });
        log::info!("R8:    {:>016X}\n", { self.r8 });
        log::info!("R9:    {:>016X}\n", { self.r9 });
        log::info!("R10:   {:>016X}\n", { self.r10 });
        log::info!("R11:   {:>016X}\n", { self.r11 });
    }
}

macro_rules! scratch_push {
    () => (llvm_asm!(
        "push rax
        push rcx
        push rdx
        push rdi
        push rsi
        push r8
        push r9
        push r10
        push r11"
        : : : : "intel", "volatile"
    ));
}

macro_rules! scratch_pop {
    () => (llvm_asm!(
        "pop r11
        pop r10
        pop r9
        pop r8
        pop rsi
        pop rdi
        pop rdx
        pop rcx
        pop rax"
        : : : : "intel", "volatile"
    ));
}

#[allow(dead_code)]
#[repr(packed)]
pub struct PreservedRegisters {
    pub r15: usize,
    pub r14: usize,
    pub r13: usize,
    pub r12: usize,
    pub rbp: usize,
    pub rbx: usize,
}

impl PreservedRegisters {
    pub fn dump(&self) {
        log::info!("RBX:   {:>016X}\n", { self.rbx });
        log::info!("RBP:   {:>016X}\n", { self.rbp });
        log::info!("R12:   {:>016X}\n", { self.r12 });
        log::info!("R13:   {:>016X}\n", { self.r13 });
        log::info!("R14:   {:>016X}\n", { self.r14 });
        log::info!("R15:   {:>016X}\n", { self.r15 });
    }
}

macro_rules! preserved_push {
    () => (llvm_asm!(
        "push rbx
        push rbp
        push r12
        push r13
        push r14
        push r15"
        : : : : "intel", "volatile"
    ));
}

macro_rules! preserved_pop {
    () => (llvm_asm!(
        "pop r15
        pop r14
        pop r13
        pop r12
        pop rbp
        pop rbx"
        : : : : "intel", "volatile"
    ));
}

#[allow(dead_code)]
#[repr(packed)]
pub struct IretRegisters {
    pub rip: usize,
    pub cs: usize,
    pub rflags: usize,
}

impl IretRegisters {
    pub fn dump(&self) {
        log::info!("RFLAG: {:>016X}\n", { self.rflags });
        log::info!("CS:    {:>016X}\n", { self.cs });
        log::info!("RIP:   {:>016X}\n", { self.rip });
    }
}

macro_rules! iret {
    () => (llvm_asm!(
        "iretq"
        : : : : "intel", "volatile"
    ));
}

#[allow(dead_code)]
#[repr(packed)]
pub struct InterruptNoErrorStack {
    pub preserved: PreservedRegisters,
    pub scratch: ScratchRegisters,
    pub iret: IretRegisters,
}

impl InterruptNoErrorStack {
    pub fn dump(&self) {
        self.iret.dump();
        self.scratch.dump();
        self.preserved.dump();
    }
}

#[allow(dead_code)]
#[repr(packed)]
pub struct InterruptErrorStack {
    pub preserved: PreservedRegisters,
    pub scratch: ScratchRegisters,
    pub code: usize,
    pub iret: IretRegisters,
}

impl InterruptErrorStack {
    pub fn dump(&self) {
        self.iret.dump();
        log::info!("CODE:  {:>016X}\n", { self.code });
        self.scratch.dump();
        self.preserved.dump();
    }
}

#[macro_export]
macro_rules! interrupt_no_error {
    ($name:ident, $stack: ident, $func:block) => {
        #[naked]
        #[no_mangle]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner($stack: &mut InterruptNoErrorStack) {
                $func
            }

            // Push scratch registers
            scratch_push!();
            preserved_push!();

            // Get reference to stack variables
            let rsp: usize;
            llvm_asm!("" : "={rsp}"(rsp) : : : "intel", "volatile");
            llvm_asm!("cld" : : : : "intel", "volatile");

            // Call inner rust function
            llvm_asm!("sub rsp, 32" : : : : "intel", "volatile");
            inner(&mut *(rsp as *mut InterruptNoErrorStack));
            llvm_asm!("add rsp, 32" : : : : "intel", "volatile");

            // Pop scratch registers and return
            preserved_pop!();
            scratch_pop!();
            iret!();
        }
    };
}

#[macro_export]
macro_rules! interrupt_error {
    ($name:ident, $stack:ident, $func:block) => {
        #[naked]
        #[no_mangle]
        pub unsafe extern fn $name () {
            #[inline(never)]
            unsafe fn inner($stack: &mut InterruptErrorStack) {
                $func
            }

            // Push scratch registers
            scratch_push!();
            preserved_push!();

            // Get reference to stack variables
            let rsp: usize;
            llvm_asm!("" : "={rsp}"(rsp) : : : "intel", "volatile");

            // Call inner rust function
            llvm_asm!("sub rsp, 40" : : : : "intel", "volatile");
            inner(&mut *(rsp as *mut InterruptErrorStack));
            llvm_asm!("add rsp, 40" : : : : "intel", "volatile");
            // Pop scratch registers, error code, and return
            preserved_pop!();
            scratch_pop!();
            llvm_asm!("add rsp, 8" : : : : "intel", "volatile");
            iret!();
        }
    };
}

interrupt_no_error!(divide_by_zero, stack, {
    log::info!("Divide by zero\n");
    stack.dump();
});

interrupt_no_error!(debug, stack, {
    log::info!("Debug trap\n");
    stack.dump();
});

interrupt_no_error!(non_maskable, stack, {
    log::info!("Non-maskable interrupt\n");
    stack.dump();
});

interrupt_no_error!(breakpoint, stack, {
    log::info!("Breakpoint trap\n");
    stack.dump();
});

interrupt_no_error!(overflow, stack, {
    log::info!("Overflow trap\n");
    stack.dump();
});

interrupt_no_error!(bound_range, stack, {
    log::info!("Bound range exceeded fault\n");
    stack.dump();
});

interrupt_no_error!(invalid_opcode, stack, {
    log::info!("Invalid opcode fault\n");
    stack.dump();
});

interrupt_no_error!(device_not_available, stack, {
    log::info!("Device not available fault\n");
    stack.dump();
});

interrupt_error!(double_fault, stack, {
    log::info!("Double fault\n");
    stack.dump();
    loop {}
});

interrupt_error!(invalid_tss, stack, {
    log::info!("Invalid TSS fault\n");
    stack.dump();
});

interrupt_error!(segment_not_present, stack, {
    log::info!("Segment not present fault\n");
    stack.dump();
});

interrupt_error!(stack_segment, stack, {
    log::info!("Stack segment fault\n");
    stack.dump();
});

interrupt_error!(protection, stack, {
    log::info!("Protection fault\n");
    stack.dump();
});

interrupt_error!(page, stack, {
    let cr2: usize;
    llvm_asm!("mov rax, cr2" : "={rax}"(cr2) : : : "intel", "volatile");
    log::info!("Page fault: {:>016X}\n", cr2);
    stack.dump();
    loop {}
});

interrupt_no_error!(fpu, stack, {
    log::info!("FPU floating point fault\n");
    stack.dump();
});

interrupt_error!(alignment_check, stack, {
    log::info!("Alignment check fault");
    stack.dump();
    loop {}
});

interrupt_no_error!(machine_check, stack, {
    log::info!("Machine check fault\n");
    stack.dump();
});

interrupt_no_error!(simd, stack, {
    log::info!("SIMD floating point fault\n");
    stack.dump();
});

interrupt_no_error!(virtualization, stack, {
    let op_code: u8 = *(stack.iret.rip as *const u8);
    match op_code {
        // IN
        0xE4 => {
            log::info!("<IN AL, IMM8>")
        }
        0xE5 => {
            log::info!("<IN EAX, IMM8>")
        }
        0xEC => {
            log::info!("<IN AL, DX>\n");
            let al = x86::io::inb((stack.scratch.rdx & 0xFFFF) as u16);
            stack.scratch.rax = (stack.scratch.rax & 0xFFFF_FFFF_FFFF_FF00_usize) | al as usize;
            stack.iret.rip += 1;
            // log::info!("Fault done\n");
            return;
        }
        0xED => {
            log::info!("<IN EAX, DX>")
        }
        // OUT
        0xE6 => {
            log::info!("<OUT IMM8, AL>")
        }
        0xE7 => {
            log::info!("<OUT IMM8, EAX>")
        }
        0xEE => {
            log::info!("<OUT DX, AL>\n");
            x86::io::outb(
                (stack.scratch.rdx & 0xFFFF) as u16,
                (stack.scratch.rax & 0xFF) as u8,
            );
            stack.iret.rip += 1;
            // log::info!("Fault done\n");
            return;
        }
        0xEF => {
            log::info!("<OUT DX, EAX>")
        }
        // Unknown
        _ => {}
    };
    log::info!("Virtualization fault\n");
    stack.dump();
    loop {}
});
