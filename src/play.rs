use fundsp::prelude::*;
use janetrs::{janet_abstract::*, IsJanetAbstract};
use kira::{
    clock::clock_info::ClockInfoProvider,
    dsp::Frame,
    manager::{backend::cpal, error::PlaySoundError, AudioManager, AudioManagerSettings},
    sound::*,
    track::TrackId,
    CommandError,
};
use once_cell::unsync::Lazy;
use ringbuf::HeapRb;
use ringbuf::{HeapConsumer, HeapProducer};
use std::ffi::*;
use std::{cell::RefCell, mem::ManuallyDrop};

thread_local! {
    pub static MANAGER: Lazy<RefCell<AudioManager<cpal::CpalBackend>>> = Lazy::new(|| RefCell::new(AudioManager::<cpal::CpalBackend>::new(AudioManagerSettings::default()).unwrap()));
}

const COMMAND_BUFFER_CAPACITY: usize = 8;
// const ERROR_BUFFER_CAPACITY: usize = 8;

#[derive(Default)]
pub enum State {
    #[default]
    Playing,
    Stopped,
    Paused,
}

pub enum Command {
    Stop,
    Pause,
    Resume,
    Reset,
}

pub struct FunData(Net64);

pub struct FunSound {
    node: Net64,
    state: State,
    rx: HeapConsumer<Command>,
}

pub struct Handle {
    tx: HeapProducer<Command>,
}

pub struct AbstractHandle(ManuallyDrop<Handle>);

impl AbstractHandle {
    pub fn new(handle: Handle) -> Self {
        Self(ManuallyDrop::new(handle))
    }
}

impl SoundData for FunData {
    type Error = ();
    type Handle = Handle;

    fn into_sound(self) -> Result<(Box<dyn Sound>, Self::Handle), Self::Error> {
        let (tx, rx) = HeapRb::new(COMMAND_BUFFER_CAPACITY).split();
        // let (err_tx, err_rx) = HeapRb::new(ERROR_BUFFER_CAPACITY).split();

        Ok((
            Box::new(FunSound {
                node: self.0,
                state: State::Playing,
                rx,
            }),
            Handle { tx },
        ))
    }
}

impl Sound for FunSound {
    fn track(&mut self) -> TrackId {
        TrackId::Main
    }

    fn on_start_processing(&mut self) {
        while let Some(command) = self.rx.pop() {
            match command {
                Command::Stop => self.state = State::Stopped,
                Command::Pause => {
                    if matches!(self.state, State::Playing) {
                        self.state = State::Paused;
                    }
                }
                Command::Resume => {
                    if matches!(self.state, State::Paused) {
                        self.state = State::Playing;
                    }
                }
                Command::Reset => {
                    if matches!(self.state, State::Playing | State::Paused) {
                        self.node.reset(None)
                    }
                }
            }
        }
    }

    fn process(&mut self, _dt: f64, _: &ClockInfoProvider) -> Frame {
        // For now, assume the device sample rate is the same as the node. We can compare the sample
        // rate to dt and reset if needed.
        let (left, right) = self.node.get_stereo();
        Frame::new(left as f32, right as f32)
    }

    fn finished(&self) -> bool {
        matches!(self.state, State::Stopped)
    }
}

/// We make no effort to reproduce kira's StreamingSoundHandle API. Just
/// stop/resume/pause/reset. Playing the same source again should reset the original (if the hash is
/// the same). Other fanciness should be handled with FunDSP's [atomic
/// variables](https://github.com/SamiPerttu/fundsp#atomic-variables) and
/// [settings](https://github.com/SamiPerttu/fundsp#settings).
impl Handle {
    pub fn stop(&mut self) -> Result<(), CommandError> {
        self.tx
            .push(Command::Stop)
            .map_err(|_| CommandError::CommandQueueFull)
    }

    pub fn pause(&mut self) -> Result<(), CommandError> {
        self.tx
            .push(Command::Pause)
            .map_err(|_| CommandError::CommandQueueFull)
    }

    pub fn resume(&mut self) -> Result<(), CommandError> {
        self.tx
            .push(Command::Resume)
            .map_err(|_| CommandError::CommandQueueFull)
    }

    pub fn reset(&mut self) -> Result<(), CommandError> {
        self.tx
            .push(Command::Reset)
            .map_err(|_| CommandError::CommandQueueFull)
    }
}

extern "C" fn handle_gc(data: *mut c_void, _len: usize) -> c_int {
    unsafe {
        let mut a = JanetAbstract::from_raw(data);
        let handle: &mut AbstractHandle = a.get_mut_unchecked();
        ManuallyDrop::drop(&mut handle.0);
    };

    0
}

// Seems like a lot of these have trait analogs...
const HANDLE_TYPE: JanetAbstractType = JanetAbstractType {
    name: "handle\0" as *const str as *const std::ffi::c_char,
    gc: Some(handle_gc),
    gcmark: None,
    get: None, // TODO methods from trait?
    put: None,
    marshal: None,
    unmarshal: None,
    tostring: None,
    compare: None,
    hash: None,
    next: None,
    call: None,
    length: None,
    bytes: None,
};

impl IsJanetAbstract for AbstractHandle {
    const SIZE: usize = std::mem::size_of::<Self>();

    #[inline]
    fn type_info() -> &'static JanetAbstractType {
        &HANDLE_TYPE
    }
}

// impl Drop for AbstractHandle {
//     fn drop(&mut self) {
//         println!("drop handle");
//     }
// }

pub fn play(net: Net64) -> Result<AbstractHandle, PlaySoundError<()>> {
    let data = FunData(net);

    MANAGER.with(|m| m.borrow_mut().play(data).map(AbstractHandle::new))
}
