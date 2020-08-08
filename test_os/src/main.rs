#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(test_os::test_runner)]
#![reexport_test_harness_main = "test_main"]  // By default, generates a main() function to test, but we have no_main

extern crate rlibc;
extern crate alloc;
use core::panic::PanicInfo;
use test_os::{println, task::{Task, keyboard, executor::Executor}};
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);  // defines any Rust function as _start() function after doing type checking

#[no_mangle]
pub unsafe extern "C" fn exit(a: i32) -> () {
    asm!("int 0x80",
         in("rax") a);// : "memory" : "intel", "volatile");
}

//#[link(name = "fibonacci", kind = "static")]
extern {
    fn fib10() -> i32;
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // BootInfo struct contains memory_map and physical_map_offset
    //  memory_map: amount of physical memory and which regions reserved for devices
    //  physical_memory_offset: start address of physical memory mapping

    use x86_64::VirtAddr;
    use test_os::{memory, allocator};

    println!("Fear is the mind killer.");

    test_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]  // Only call test_main in test contexts, since it is not generated on a normal run
    test_main();

    unsafe { fib10() }; // make an exit syscall with exit code 55

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    // example_task() returns a future, which is then wrapped in a Task to move
    // it to the heap and pin it, and executor.spawn() adds it to the task_queue

    executor.spawn(Task::new(keyboard::print_keypresses()));

    executor.run();
    // pops the task from the front of the task_queue
    // creates a RawWaker for the task, converts it to a Waker, then creates a Context instance
    // calls the poll() method on the future of the task using the Context just created
    // example_task does not wait for anything, so it runs directly until the end
    // example_task directly returns Poll::Ready, so is not added back to the task queue

    // executor.run() never returns, so no need for hlt_loop()
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    test_os::hlt_loop();
}

// Panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_os::test_panic_handler(info)
}

#[test_case]
fn test_main() {
    assert_eq!(1, 1);
}
