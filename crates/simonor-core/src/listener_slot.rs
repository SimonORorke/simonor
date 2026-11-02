use std::sync::{Arc, Mutex, Weak};

/// A reusable holder for an application-specific ("semantic") event listener.
///
/// This factors out the common "semantic listening" pattern used throughout the simonor
/// applications: a raw MIDI-handling type (such as a `MidiReceiver`) raises application-specific
/// events to a listener it knows nothing about beyond a trait, such as `OctaveShiftListener` in
/// `octave-up-down` or `ContinuumProtocolListener` in `pitchgrid_continuum`. The listener is
/// typically the `Presenter`, held weakly to avoid a reference cycle (the `Presenter` normally
/// owns, directly or indirectly, the raw MIDI-handling type), and is usually set once, from
/// `Presenter::init`, via `set`.
///
/// `L` is the (generally `?Sized`) listener trait, e.g. `dyn OctaveShiftListener`.
pub struct ListenerSlot<L: ?Sized> {
    listener: Mutex<Option<Weak<L>>>,
}

impl<L: ?Sized> ListenerSlot<L> {
    pub fn new() -> Self {
        Self {
            listener: Mutex::new(None),
        }
    }

    /// Records the (weak) listener. Typically called once, from `Presenter::init`.
    pub fn set(&self, listener: Weak<L>) {
        *self.listener.lock().unwrap() = Some(listener);
    }

    /// The listener, if one has been set and it is still alive.
    pub fn get(&self) -> Option<Arc<L>> {
        self.listener.lock().unwrap().as_ref().and_then(Weak::upgrade)
    }
}

impl<L: ?Sized> Default for ListenerSlot<L> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: ?Sized + Send + Sync + 'static> ListenerSlot<L> {
    /// Raises an event to the listener, if it is still alive, on a background thread, mirroring
    /// the existing `rayon::spawn` usage for semantic events. `raise` is called with the upgraded
    /// listener only if it is still alive; nothing happens otherwise.
    pub fn notify<F>(&self, raise: F)
    where
        F: FnOnce(Arc<L>) + Send + 'static,
    {
        if let Some(listener) = self.get() {
            rayon::spawn(move || raise(listener));
        }
    }
}

/// Generates the boilerplate `impl` that forwards a semantic listener trait's methods from
/// `Mutex<T>` to the locked `T`, so that an `Arc<Mutex<T>>` (such as a `SharedPresenter`) can
/// itself be used as `Arc<dyn Listener>`.
///
/// Usage:
/// ```ignore
/// simonor::impl_mutex_listener!(OctaveShiftListener for Presenter {
///     fn on_octave_shift_changed(&self, info_message: String);
/// });
/// ```
#[macro_export]
macro_rules! impl_mutex_listener {
    ($trait_name:ident for $ty:ty { $(fn $method:ident(&self $(, $arg:ident : $arg_ty:ty)* $(,)?);)+ }) => {
        impl $trait_name for ::std::sync::Mutex<$ty> {
            $(
                fn $method(&self $(, $arg : $arg_ty)*) {
                    self.lock().unwrap().$method($($arg),*);
                }
            )+
        }
    };
}
