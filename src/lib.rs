#![no_std]
#![cfg_attr(test, no_main)] // test为了集成测试
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
pub mod serial;
pub mod vga_buffer;

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]//条件编译
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success)
}

// our panic handler in test mode
pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where 
    T: Fn()
{
    fn run(&self){
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

// 最终qemu的exit status = (value << 1) | 1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]  //将其内存布局 与 u32 相当,也就是QemuExitCode::Success 相当于 u32
pub enum QemuExitCode {
    // cargo test 会将0以外的退出码视为faile, 需要 bootimage 帮忙转一下
    Success = 0x10, //不能是0 不然跟 qemu默认退出码 1 一样了
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode){
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port= Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

