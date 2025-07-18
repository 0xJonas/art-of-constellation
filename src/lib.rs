use std::{cell::RefCell, panic::{set_hook, PanicHookInfo}};

use skylite_core::SkyliteProject;
use wasm4_target::{trace, w4alloc::W4Alloc, Wasm4Target};

use crate::aoc::Aoc;

mod star;
mod sky;
mod line;
mod util;

#[global_allocator]
static ALLOC: W4Alloc = W4Alloc::new();

#[skylite_proc::skylite_project("./project/project.scm", Wasm4Target)]
mod aoc {
    use wasm4_target::Wasm4Target;
    use crate::sky::Sky;
}

thread_local! {
    static GAME: RefCell<Option<Aoc>> = RefCell::new(None);
}

#[cfg(debug_assertions)]
fn panic_hook(info: &PanicHookInfo) {
    trace(format!("Cart panicked: {:?}\n", info.payload()));
    if let Some(location) = info.location() {
        trace(format!("  at {}:{}:{}\n", location.file(), location.line(), location.column()));
    }
}

#[unsafe(no_mangle)]
fn start() {
    #[cfg(debug_assertions)]
    set_hook(Box::new(panic_hook));

    GAME.with(|game| {
        let target = Wasm4Target::new();
        let _ = game.replace(Some(Aoc::new(target)));
    });
}

#[unsafe(no_mangle)]
fn update() {
    GAME.with(|game| {
        game.borrow_mut().as_mut().unwrap().update();
        game.borrow_mut().as_mut().unwrap().render();
    });
}
