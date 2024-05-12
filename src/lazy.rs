use std::{fmt, mem};
use std::cell::UnsafeCell;
use std::fmt::Pointer;
use std::ops::Deref;



enum State<T, F> {
    Uninit(F),
    Init(T),
    Poisoned,
}


pub struct Lazy<T, F = fn() -> T> {
    state: UnsafeCell<State<T, F>>,
}

impl<T, F: FnOnce() -> T> Lazy<T, F> {

    #[inline]
    pub const fn init(v: T) -> Lazy<T, F> {
        Lazy { state: UnsafeCell::new(State::Init(v)) }
    }
    #[inline]
    pub const fn new(f: F) -> Lazy<T, F> { Lazy { state: UnsafeCell::new(State::Uninit(f)) } }


    pub fn into_inner(self) -> Result<T, F> {
        match self.state.into_inner() {
            State::Init(data) => Ok(data),
            State::Uninit(f) => Err(f),
            State::Poisoned => panic!("Lazy instance has previously been poisoned"),
        }
    }


    #[inline]
    pub fn force(this: &Lazy<T, F>) -> &T {
        // SAFETY:
        // This invalidates any mutable references to the data. The resulting
        // reference lives either until the end of the borrow of `this` (in the
        // initialized case) or is invalidated in `really_init` (in the
        // uninitialized case; `really_init` will create and return a fresh reference).
        let state = unsafe { &*this.state.get() };
        match state {
            State::Init(data) => data,
            // SAFETY: The state is uninitialized.
            State::Uninit(_) => unsafe { Lazy::really_init(this) },
            State::Poisoned => panic!("Lazy has previously been poisoned"),
        }
    }

    #[cold]
    unsafe fn really_init(this: &Lazy<T, F>) -> &T {
        // SAFETY:
        // This function is only called when the state is uninitialized,
        // so no references to `state` can exist except for the reference
        // in `force`, which is invalidated here and not accessed again.
        let state = unsafe { &mut *this.state.get() };
        // Temporarily mark the state as poisoned. This prevents reentrant
        // accesses and correctly poisons the cell if the closure panicked.
        let State::Uninit(f) = mem::replace(state, State::Poisoned) else { unreachable!() };

        let data = f();

        // SAFETY:
        // If the closure accessed the cell through something like a reentrant
        // mutex, but caught the panic resulting from the state being poisoned,
        // the mutable borrow for `state` will be invalidated, so we need to
        // go through the `UnsafeCell` pointer here. The state can only be
        // poisoned at this point, so using `write` to skip the destructor
        // of `State` should help the optimizer.
        unsafe { this.state.get().write(State::Init(data)) };

        // SAFETY:
        // The previous references were invalidated by the `write` call above,
        // so do a new shared borrow of the state instead.
        let state = unsafe { &*this.state.get() };
        let State::Init(data) = state else { unreachable!() };
        data
    }
}

impl<T, F> Lazy<T, F> {
    #[inline]
    fn get(&self) -> Option<&T> {
        // SAFETY:
        // This is sound for the same reason as in `force`: once the state is
        // initialized, it will not be mutably accessed again, so this reference
        // will stay valid for the duration of the borrow to `self`.
        let state = unsafe { &*self.state.get() };
        match state {
            State::Init(data) => Some(data),
            _ => None,
        }
    }
}

impl<T, F: FnOnce() -> T> Deref for Lazy<T, F> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        Lazy::force(self)
    }
}

impl<T: Default> Default for Lazy<T> {
    /// Creates a new lazy value using `Default` as the initializing function.
    #[inline]
    fn default() -> Lazy<T> {
        Lazy::new(T::default)
    }
}

impl<T: core::fmt::Debug, F: FnOnce() -> T> core::fmt::Debug for Lazy<T, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.deref().fmt(f)
    }
}
